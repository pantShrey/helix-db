// DEFAULT CODE
// use helix_db::helix_engine::traversal_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }

use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, embed_async, exclude_field, field_addition_from_old_field, field_addition_from_value,
    field_remapping, field_type_cast,
    helix_engine::{
        traversal_core::{
            config::{Config, GraphConfig, VectorConfig},
            ops::{
                bm25::search_bm25::SearchBM25Adapter,
                g::G,
                in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
                out::{
                    from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter,
                    out_e::OutEdgesAdapter,
                },
                source::{
                    add_e::{AddEAdapter, EdgeType},
                    add_n::AddNAdapter,
                    e_from_id::EFromIdAdapter,
                    e_from_type::EFromTypeAdapter,
                    n_from_id::NFromIdAdapter,
                    n_from_index::NFromIndexAdapter,
                    n_from_type::NFromTypeAdapter,
                },
                util::{
                    count::CountAdapter, dedup::DedupAdapter, drop::Drop, exist::Exist,
                    filter_mut::FilterMut, filter_ref::FilterRefAdapter, map::MapAdapter,
                    order::OrderByAdapter, paths::ShortestPathAdapter, props::PropsAdapter,
                    range::RangeAdapter, update::UpdateAdapter,
                },
                vectors::{
                    brute_force_search::BruteForceSearchVAdapter, insert::InsertVAdapter,
                    search::SearchVAdapter,
                },
            },
            traversal_value::{Traversable, TraversalValue},
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        mcp::mcp::{MCPHandler, MCPHandlerSubmission, MCPToolInput},
        router::router::{HandlerInput, IoContFn},
    },
    identifier_remapping, node_matches, props,
    protocol::{
        format::Format,
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::{
            Value,
            casting::{CastType, cast},
        },
    },
    traversal_remapping,
    utils::{
        count::Count,
        filterable::Filterable,
        id::ID,
        items::{Edge, Node},
    },
    value_remapping,
};
use helix_macros::{handler, mcp_handler, migration, tool_call};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub fn config() -> Option<Config> {
    return Some(Config {
vector_config: Some(VectorConfig {
m: Some(16),
ef_construction: Some(128),
ef_search: Some(768),
preload: true,
}),
graph_config: Some(GraphConfig {
secondary_indices: Some(vec!["uuid".to_string(), "username".to_string()]),
}),

db_max_size_gb: Some(100),
mcp: Some(true),
bm25: Some(true),
schema: Some(r#"{
  "schema": {
    "nodes": [
      {
        "name": "Comment_Cluster2",
        "properties": {
          "uuid": "String",
          "parent_uuid": "String",
          "created_at": "Date",
          "id": "ID",
          "text": "String",
          "klive": "I64"
        }
      },
      {
        "name": "Event_Cluster1",
        "properties": {
          "invalidated_by": "String",
          "created_at": "Date",
          "invalid_at": "Date",
          "triplets": "Array(String)",
          "statement_type": "String",
          "expired_at": "Date",
          "statement": "String",
          "valid_at": "Date",
          "temporal_type": "String",
          "uuid": "String",
          "id": "ID",
          "chunk_uuid": "String"
        }
      },
      {
        "name": "Entity_Cluster1",
        "properties": {
          "resolved_id": "String",
          "name": "String",
          "entity_type": "String",
          "created_at": "Date",
          "id": "ID",
          "description": "String",
          "uuid": "String",
          "event_uuid": "String"
        }
      },
      {
        "name": "User_Cluster2",
        "properties": {
          "created_at": "Date",
          "username": "String",
          "id": "ID"
        }
      },
      {
        "name": "Story_Cluster1",
        "properties": {
          "title": "String",
          "created_at": "Date",
          "url": "String",
          "username": "String",
          "uuid": "String",
          "text": "String",
          "score": "I64",
          "id": "ID",
          "klive": "I64"
        }
      },
      {
        "name": "Chunk_Cluster1",
        "properties": {
          "uuid": "String",
          "text": "String",
          "metadata": "String",
          "id": "ID",
          "story_uuid": "String"
        }
      },
      {
        "name": "Triplet_Cluster1",
        "properties": {
          "object_uuid": "String",
          "id": "ID",
          "subject_name": "String",
          "created_at": "Date",
          "uuid": "String",
          "object_name": "String",
          "value": "String",
          "predicate": "String",
          "subject_uuid": "String",
          "event_uuid": "String"
        }
      },
      {
        "name": "Story_Cluster2",
        "properties": {
          "id": "ID",
          "url": "String",
          "score": "I64",
          "title": "String",
          "klive": "I64",
          "uuid": "String",
          "created_at": "Date",
          "text": "String"
        }
      },
      {
        "name": "Comment_Cluster1",
        "properties": {
          "uuid": "String",
          "created_at": "Date",
          "id": "ID",
          "parent_uuid": "String",
          "text": "String",
          "username": "String",
          "klive": "I64"
        }
      }
    ],
    "vectors": [
      {
        "name": "EventEmbedding_Cluster1",
        "properties": {
          "embedding": "Array(F64)",
          "id": "ID"
        }
      },
      {
        "name": "StoryEmbedding_Cluster2",
        "properties": {
          "id": "ID",
          "content": "String"
        }
      },
      {
        "name": "CommentEmbedding_Cluster2",
        "properties": {
          "id": "ID",
          "content": "String"
        }
      }
    ],
    "edges": [
      {
        "name": "Triplet_to_Object_Cluster1",
        "from": "Triplet_Cluster1",
        "to": "Entity_Cluster1",
        "properties": {}
      },
      {
        "name": "Chunk_to_Event_Cluster1",
        "from": "Chunk_Cluster1",
        "to": "Event_Cluster1",
        "properties": {}
      },
      {
        "name": "Invalidated_By_Cluster1",
        "from": "Event_Cluster1",
        "to": "Event_Cluster1",
        "properties": {}
      },
      {
        "name": "User_to_Story_Cluster2",
        "from": "User_Cluster2",
        "to": "Story_Cluster2",
        "properties": {}
      },
      {
        "name": "Comment_to_Comment_Cluster1",
        "from": "Comment_Cluster1",
        "to": "Comment_Cluster1",
        "properties": {}
      },
      {
        "name": "Event_to_Triplet_Cluster1",
        "from": "Event_Cluster1",
        "to": "Triplet_Cluster1",
        "properties": {}
      },
      {
        "name": "Story_to_Comment_Cluster2",
        "from": "Story_Cluster2",
        "to": "Comment_Cluster2",
        "properties": {
          "story_uuid": "String",
          "comment_uuid": "String"
        }
      },
      {
        "name": "Event_to_Entity_Cluster1",
        "from": "Event_Cluster1",
        "to": "Entity_Cluster1",
        "properties": {}
      },
      {
        "name": "Comment_to_Comment_Cluster2",
        "from": "Comment_Cluster2",
        "to": "Comment_Cluster2",
        "properties": {}
      },
      {
        "name": "Triplet_to_Subject_Cluster1",
        "from": "Triplet_Cluster1",
        "to": "Entity_Cluster1",
        "properties": {}
      },
      {
        "name": "Story_to_Chunk_Cluster1",
        "from": "Story_Cluster1",
        "to": "Chunk_Cluster1",
        "properties": {}
      },
      {
        "name": "Story_to_StoryEmbedding_Cluster2",
        "from": "Story_Cluster2",
        "to": "StoryEmbedding_Cluster2",
        "properties": {}
      },
      {
        "name": "Event_to_Embedding_Cluster1",
        "from": "Event_Cluster1",
        "to": "EventEmbedding_Cluster1",
        "properties": {}
      },
      {
        "name": "Resolved_Cluster1",
        "from": "Entity_Cluster1",
        "to": "Entity_Cluster1",
        "properties": {}
      },
      {
        "name": "Comment_to_CommentEmbedding_Cluster2",
        "from": "Comment_Cluster2",
        "to": "CommentEmbedding_Cluster2",
        "properties": {}
      },
      {
        "name": "Story_to_Comment_Cluster1",
        "from": "Story_Cluster1",
        "to": "Comment_Cluster1",
        "properties": {
          "story_uuid": "String",
          "comment_uuid": "String"
        }
      },
      {
        "name": "User_to_Comments_Cluster2",
        "from": "User_Cluster2",
        "to": "Comment_Cluster2",
        "properties": {}
      }
    ]
  },
  "queries": [
    {
      "name": "insert_story_Cluster1",
      "parameters": {
        "created_at": "Date",
        "text": "String",
        "uuid": "String",
        "score": "I64",
        "klive": "I64",
        "title": "String",
        "url": "String",
        "username": "String"
      },
      "returns": [
        "story"
      ]
    },
    {
      "name": "get_comments_by_story_uuid_Cluster1",
      "parameters": {
        "story_uuid": "String"
      },
      "returns": [
        "comments"
      ]
    },
    {
      "name": "get_event_chunk_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": []
    },
    {
      "name": "get_triplets_by_object_uuid_Cluster1",
      "parameters": {
        "object_uuid": "String"
      },
      "returns": [
        "triplets"
      ]
    },
    {
      "name": "insert_story_Cluster2_batch",
      "parameters": {
        "stories": "Array({by: Stringurl: Stringcreated_at: Dateuuid: Stringtext: Stringtitle: Stringklive: I64score: I64})"
      },
      "returns": []
    },
    {
      "name": "insert_sub_comment_Cluster1",
      "parameters": {
        "parent_uuid": "String",
        "uuid": "String",
        "text": "String",
        "created_at": "Date",
        "klive": "I64",
        "username": "String"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "add_story_embedding_Cluster2_batch",
      "parameters": {
        "story_embeddings": "Array({story_uuid: Stringcontent: Stringembedding: Array(F64)})"
      },
      "returns": []
    },
    {
      "name": "drop_all_story_and_embeddings_Cluster2",
      "parameters": {},
      "returns": []
    },
    {
      "name": "get_triplets_by_subject_uuid_Cluster1",
      "parameters": {
        "subject_uuid": "String"
      },
      "returns": [
        "triplets"
      ]
    },
    {
      "name": "add_comment_embedding_Cluster2",
      "parameters": {
        "comment_uuid": "String",
        "embedding": "Array(F64)",
        "content": "String"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "add_comment_embedding_Cluster2_batch",
      "parameters": {
        "comment_embeddings": "Array({content: Stringcomment_uuid: Stringembedding: Array(F64)})"
      },
      "returns": []
    },
    {
      "name": "get_all_comments_Cluster2",
      "parameters": {},
      "returns": [
        "comments"
      ]
    },
    {
      "name": "drop_all_tables_Cluster1",
      "parameters": {},
      "returns": []
    },
    {
      "name": "drop_all_Cluster2",
      "parameters": {},
      "returns": []
    },
    {
      "name": "insert_user_Cluster2",
      "parameters": {
        "username": "String"
      },
      "returns": [
        "user"
      ]
    },
    {
      "name": "get_story_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "story"
      ]
    },
    {
      "name": "get_chunk_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "chunk"
      ]
    },
    {
      "name": "expire_event_Cluster1",
      "parameters": {
        "uuid": "String",
        "expired_at": "Date"
      },
      "returns": [
        "event"
      ]
    },
    {
      "name": "drop_all_users_Cluster2",
      "parameters": {},
      "returns": []
    },
    {
      "name": "get_all_events_without_embeddings_Cluster1",
      "parameters": {},
      "returns": [
        "events"
      ]
    },
    {
      "name": "get_comments_by_story_uuid_Cluster2",
      "parameters": {
        "story_uuid": "String"
      },
      "returns": [
        "comments"
      ]
    },
    {
      "name": "get_triplet_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "triplet"
      ]
    },
    {
      "name": "insert_comment_Cluster1",
      "parameters": {
        "parent_uuid": "String",
        "username": "String",
        "text": "String",
        "uuid": "String",
        "created_at": "Date",
        "klive": "I64"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "insert_user_Cluster2_batch",
      "parameters": {
        "users": "Array({username: String})"
      },
      "returns": []
    },
    {
      "name": "connect_user_to_story_Cluster2",
      "parameters": {
        "story_uuid": "String",
        "username": "String"
      },
      "returns": [
        "edge"
      ]
    },
    {
      "name": "get_all_stories_Cluster2",
      "parameters": {},
      "returns": [
        "stories"
      ]
    },
    {
      "name": "update_triplet_object_Cluster1",
      "parameters": {
        "object_uuid": "String",
        "uuid": "String"
      },
      "returns": [
        "triplet"
      ]
    },
    {
      "name": "get_all_entities_Cluster1",
      "parameters": {},
      "returns": [
        "entities"
      ]
    },
    {
      "name": "insert_story_Cluster2",
      "parameters": {
        "klive": "I64",
        "uuid": "String",
        "score": "I64",
        "url": "String",
        "created_at": "Date",
        "by": "String",
        "text": "String",
        "title": "String"
      },
      "returns": [
        "story"
      ]
    },
    {
      "name": "insert_event_Cluster1",
      "parameters": {
        "valid_at": "Date",
        "created_at": "Date",
        "triplets": "Array(String)",
        "uuid": "String",
        "chunk_uuid": "String",
        "embedding": "Array(F64)",
        "statement_type": "String",
        "statement": "String",
        "temporal_type": "String"
      },
      "returns": [
        "event"
      ]
    },
    {
      "name": "get_triplet_as_object_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "triplets"
      ]
    },
    {
      "name": "insert_sub_comment_Cluster2_batch",
      "parameters": {
        "sub_comments": "Array({klive: I64parent_uuid: Stringuuid: Stringcreated_at: Datetext: Stringusername: String})"
      },
      "returns": []
    },
    {
      "name": "vector_search_events_Cluster1",
      "parameters": {
        "query_embedding": "Array(F64)",
        "k": "I32"
      },
      "returns": [
        "events",
        "triplets",
        "entities",
        "chunks"
      ]
    },
    {
      "name": "get_stories_mentioning_entity_as_subject_Cluster1",
      "parameters": {
        "entity_uuid": "String"
      },
      "returns": [
        "stories",
        "chunks",
        "events",
        "triplets"
      ]
    },
    {
      "name": "get_all_triplets_Cluster1",
      "parameters": {},
      "returns": [
        "triplets"
      ]
    },
    {
      "name": "insert_entity_Cluster1",
      "parameters": {
        "entity_type": "String",
        "created_at": "Date",
        "uuid": "String",
        "description": "String",
        "event_uuid": "String",
        "name": "String"
      },
      "returns": [
        "entity"
      ]
    },
    {
      "name": "get_chunks_by_story_uuid_Cluster1",
      "parameters": {
        "story_uuid": "String"
      },
      "returns": [
        "chunks"
      ]
    },
    {
      "name": "resolve_entity_Cluster1",
      "parameters": {
        "resolved_id": "String",
        "uuid": "String"
      },
      "returns": [
        "new_entity"
      ]
    },
    {
      "name": "get_entities_in_story_Cluster1",
      "parameters": {
        "story_uuid": "String"
      },
      "returns": [
        "subject_entities",
        "object_entities",
        "triplets",
        "events",
        "chunks"
      ]
    },
    {
      "name": "insert_comment_Cluster2",
      "parameters": {
        "klive": "I64",
        "parent_uuid": "String",
        "text": "String",
        "uuid": "String",
        "created_at": "Date",
        "username": "String"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "get_entity_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "entity"
      ]
    },
    {
      "name": "invalidate_event_Cluster1",
      "parameters": {
        "invalidated_by": "String",
        "invalid_at": "Date",
        "uuid": "String"
      },
      "returns": [
        "event"
      ]
    },
    {
      "name": "insert_triplet_Cluster1",
      "parameters": {
        "value": "String",
        "subject_name": "String",
        "uuid": "String",
        "object_uuid": "String",
        "predicate": "String",
        "object_name": "String",
        "created_at": "Date",
        "subject_uuid": "String",
        "event_uuid": "String"
      },
      "returns": [
        "triplet"
      ]
    },
    {
      "name": "get_triplet_as_subject_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "triplets"
      ]
    },
    {
      "name": "update_event_chunk_Cluster1",
      "parameters": {
        "uuid": "String",
        "chunk_uuid": "String"
      },
      "returns": [
        "event"
      ]
    },
    {
      "name": "add_story_embedding_Cluster2",
      "parameters": {
        "embedding": "Array(F64)",
        "content": "String",
        "story_uuid": "String"
      },
      "returns": [
        "story"
      ]
    },
    {
      "name": "drop_all_comments_Cluster2",
      "parameters": {
        "k": "I32"
      },
      "returns": []
    },
    {
      "name": "get_all_stories_Cluster1",
      "parameters": {},
      "returns": [
        "stories"
      ]
    },
    {
      "name": "get_sub_comments_by_parent_uuid_Cluster1",
      "parameters": {
        "parent_uuid": "String"
      },
      "returns": [
        "comments"
      ]
    },
    {
      "name": "get_entity_by_resolved_id_Cluster1",
      "parameters": {
        "resolved_id": "String"
      },
      "returns": [
        "entities"
      ]
    },
    {
      "name": "get_event_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "event",
        "embedding"
      ]
    },
    {
      "name": "update_event_Cluster1",
      "parameters": {
        "statement": "String",
        "triplets": "Array(String)",
        "statement_type": "String",
        "embedding": "Array(F64)",
        "temporal_type": "String",
        "valid_at": "Date",
        "created_at": "Date",
        "chunk_uuid": "String",
        "uuid": "String"
      },
      "returns": [
        "event"
      ]
    },
    {
      "name": "remove_entity_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": []
    },
    {
      "name": "insert_chunk_Cluster1",
      "parameters": {
        "uuid": "String",
        "story_uuid": "String",
        "metadata": "String",
        "text": "String"
      },
      "returns": [
        "chunk"
      ]
    },
    {
      "name": "get_stories_by_predicate_Cluster1",
      "parameters": {
        "predicate": "String"
      },
      "returns": [
        "stories",
        "triplets",
        "events",
        "chunks"
      ]
    },
    {
      "name": "get_story_by_uuid_Cluster2",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "story"
      ]
    },
    {
      "name": "connect_user_to_story_Cluster2_batch",
      "parameters": {
        "user_story_pairs": "Array({username: Stringstory_uuid: String})"
      },
      "returns": []
    },
    {
      "name": "get_stories_mentioning_entity_as_object_Cluster1",
      "parameters": {
        "entity_uuid": "String"
      },
      "returns": [
        "stories",
        "chunks",
        "events",
        "triplets"
      ]
    },
    {
      "name": "get_user_by_username_Cluster2",
      "parameters": {
        "username": "String"
      },
      "returns": [
        "user"
      ]
    },
    {
      "name": "connect_user_to_comment_Cluster2",
      "parameters": {
        "username": "String",
        "comment_uuid": "String"
      },
      "returns": [
        "edge"
      ]
    },
    {
      "name": "get_story_by_title_Cluster1",
      "parameters": {
        "title": "String"
      },
      "returns": [
        "stories"
      ]
    },
    {
      "name": "get_comment_by_uuid_Cluster2",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "get_sub_comments_by_parent_uuid_Cluster2",
      "parameters": {
        "parent_uuid": "String"
      },
      "returns": [
        "sub_comments"
      ]
    },
    {
      "name": "has_events_Cluster1",
      "parameters": {},
      "returns": [
        "events"
      ]
    },
    {
      "name": "search_entity_with_stories_by_name_Cluster1",
      "parameters": {
        "entity_name": "String"
      },
      "returns": [
        "entities",
        "subject_stories",
        "object_stories",
        "subject_triplets",
        "object_triplets"
      ]
    },
    {
      "name": "get_all_users_Cluster2",
      "parameters": {},
      "returns": [
        "users"
      ]
    },
    {
      "name": "get_comment_by_uuid_Cluster1",
      "parameters": {
        "uuid": "String"
      },
      "returns": [
        "comment"
      ]
    },
    {
      "name": "get_event_embedding_by_event_uuid_Cluster1",
      "parameters": {
        "event_uuid": "String"
      },
      "returns": [
        "embedding"
      ]
    },
    {
      "name": "connect_user_to_comment_Cluster2_batch",
      "parameters": {
        "user_comment_pairs": "Array({comment_uuid: Stringusername: String})"
      },
      "returns": []
    },
    {
      "name": "insert_comment_Cluster2_batch",
      "parameters": {
        "comments": "Array({created_at: Dateuuid: Stringparent_uuid: Stringusername: Stringtext: Stringklive: I64})"
      },
      "returns": []
    },
    {
      "name": "count_all_stories_Cluster2",
      "parameters": {},
      "returns": [
        "stories"
      ]
    },
    {
      "name": "update_triplet_subject_Cluster1",
      "parameters": {
        "uuid": "String",
        "subject_uuid": "String"
      },
      "returns": [
        "triplet"
      ]
    },
    {
      "name": "update_entity_resolved_id_Cluster1",
      "parameters": {
        "uuid": "String",
        "resolved_id": "String"
      },
      "returns": [
        "entity"
      ]
    },
    {
      "name": "search_similar_stories_Cluster2",
      "parameters": {
        "query_embedding": "Array(F64)",
        "k": "I64"
      },
      "returns": [
        "stories"
      ]
    },
    {
      "name": "insert_sub_comment_Cluster2",
      "parameters": {
        "uuid": "String",
        "created_at": "Date",
        "username": "String",
        "klive": "I64",
        "parent_uuid": "String",
        "text": "String"
      },
      "returns": [
        "comment"
      ]
    }
  ]
}"#.to_string()),
embedding_model: Some("text-embedding-3-small".to_string()),
graphvis_node_label: Some("".to_string()),
});
}

pub struct Story_Cluster1 {
    pub uuid: String,
    pub username: String,
    pub title: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub score: i64,
    pub klive: i64,
}

pub struct Comment_Cluster1 {
    pub uuid: String,
    pub username: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}

pub struct Chunk_Cluster1 {
    pub uuid: String,
    pub story_uuid: String,
    pub text: String,
    pub metadata: String,
}

pub struct Event_Cluster1 {
    pub uuid: String,
    pub chunk_uuid: String,
    pub statement: String,
    pub triplets: Vec<String>,
    pub statement_type: String,
    pub temporal_type: String,
    pub created_at: DateTime<Utc>,
    pub valid_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
    pub invalid_at: DateTime<Utc>,
    pub invalidated_by: String,
}

pub struct Triplet_Cluster1 {
    pub uuid: String,
    pub event_uuid: String,
    pub subject_name: String,
    pub subject_uuid: String,
    pub predicate: String,
    pub object_name: String,
    pub object_uuid: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}

pub struct Entity_Cluster1 {
    pub uuid: String,
    pub event_uuid: String,
    pub name: String,
    pub entity_type: String,
    pub description: String,
    pub resolved_id: String,
    pub created_at: DateTime<Utc>,
}

pub struct Story_Cluster2 {
    pub uuid: String,
    pub title: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub score: i64,
    pub klive: i64,
}

pub struct Comment_Cluster2 {
    pub uuid: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}

pub struct User_Cluster2 {
    pub username: String,
    pub created_at: DateTime<Utc>,
}

pub struct Story_to_Comment_Cluster1 {
    pub from: Story_Cluster1,
    pub to: Comment_Cluster1,
    pub story_uuid: String,
    pub comment_uuid: String,
}

pub struct Comment_to_Comment_Cluster1 {
    pub from: Comment_Cluster1,
    pub to: Comment_Cluster1,
}

pub struct Story_to_Chunk_Cluster1 {
    pub from: Story_Cluster1,
    pub to: Chunk_Cluster1,
}

pub struct Chunk_to_Event_Cluster1 {
    pub from: Chunk_Cluster1,
    pub to: Event_Cluster1,
}

pub struct Event_to_Embedding_Cluster1 {
    pub from: Event_Cluster1,
    pub to: EventEmbedding_Cluster1,
}

pub struct Invalidated_By_Cluster1 {
    pub from: Event_Cluster1,
    pub to: Event_Cluster1,
}

pub struct Event_to_Triplet_Cluster1 {
    pub from: Event_Cluster1,
    pub to: Triplet_Cluster1,
}

pub struct Event_to_Entity_Cluster1 {
    pub from: Event_Cluster1,
    pub to: Entity_Cluster1,
}

pub struct Triplet_to_Subject_Cluster1 {
    pub from: Triplet_Cluster1,
    pub to: Entity_Cluster1,
}

pub struct Triplet_to_Object_Cluster1 {
    pub from: Triplet_Cluster1,
    pub to: Entity_Cluster1,
}

pub struct Resolved_Cluster1 {
    pub from: Entity_Cluster1,
    pub to: Entity_Cluster1,
}

pub struct Story_to_Comment_Cluster2 {
    pub from: Story_Cluster2,
    pub to: Comment_Cluster2,
    pub story_uuid: String,
    pub comment_uuid: String,
}

pub struct Comment_to_Comment_Cluster2 {
    pub from: Comment_Cluster2,
    pub to: Comment_Cluster2,
}

pub struct Comment_to_CommentEmbedding_Cluster2 {
    pub from: Comment_Cluster2,
    pub to: CommentEmbedding_Cluster2,
}

pub struct Story_to_StoryEmbedding_Cluster2 {
    pub from: Story_Cluster2,
    pub to: StoryEmbedding_Cluster2,
}

pub struct User_to_Story_Cluster2 {
    pub from: User_Cluster2,
    pub to: Story_Cluster2,
}

pub struct User_to_Comments_Cluster2 {
    pub from: User_Cluster2,
    pub to: Comment_Cluster2,
}

pub struct EventEmbedding_Cluster1 {
    pub embedding: Vec<f64>,
}

pub struct CommentEmbedding_Cluster2 {
    pub content: String,
}

pub struct StoryEmbedding_Cluster2 {
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_story_Cluster1Input {
    pub uuid: String,
    pub username: String,
    pub title: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub score: i64,
    pub klive: i64,
}
#[handler]
pub fn insert_story_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_story_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let story = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Story_Cluster1", Some(props! { "uuid" => &data.uuid, "created_at" => &data.created_at, "username" => &data.username, "klive" => &data.klive, "text" => &data.text, "url" => &data.url, "title" => &data.title, "score" => &data.score }), Some(&["uuid"])).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "story".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            story.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_comments_by_story_uuid_Cluster1Input {
    pub story_uuid: String,
}
#[handler]
pub fn get_comments_by_story_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_comments_by_story_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.story_uuid)
        .collect_to_obj();
    let comments = G::new_from(Arc::clone(&db), &txn, story.clone())
        .out("Story_to_Comment_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comments".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            comments.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_event_chunk_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_event_chunk_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_event_chunk_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let event = G::new(Arc::clone(&db), &txn)
        .n_from_index("Event_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            G::new_from(Arc::clone(&db), &txn, event.clone())
                .check_property("chunk_uuid")
                .collect_to_obj()
                .clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_triplets_by_object_uuid_Cluster1Input {
    pub object_uuid: String,
}
#[handler]
pub fn get_triplets_by_object_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_triplets_by_object_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let triplets = G::new(Arc::clone(&db), &txn)
        .n_from_type("Triplet_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("object_uuid")
                    .map_value_or(false, |v| *v == data.object_uuid.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_story_Cluster2_batchInput {
    pub stories: Vec<storiesData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct storiesData {
    pub by: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub uuid: String,
    pub text: String,
    pub title: String,
    pub klive: i64,
    pub score: i64,
}
#[handler]
pub fn insert_story_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_story_Cluster2_batchInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for storiesData {
        uuid,
        title,
        text,
        created_at,
        url,
        by,
        score,
        klive,
    } in &data.stories
    {
        let story = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Story_Cluster2", Some(props! { "text" => &text, "score" => &score, "created_at" => &created_at, "uuid" => &uuid, "klive" => &klive, "title" => &title, "url" => &url }), Some(&["uuid"])).collect_to_obj();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_index("User_Cluster2", "username", &by)
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "User_to_Story_Cluster2",
                None,
                user.id(),
                story.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_sub_comment_Cluster1Input {
    pub uuid: String,
    pub username: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}
#[handler]
pub fn insert_sub_comment_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_sub_comment_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster1", Some(props! { "text" => &data.text, "created_at" => &data.created_at, "klive" => &data.klive, "username" => &data.username, "parent_uuid" => &data.parent_uuid, "uuid" => &data.uuid }), Some(&["uuid"])).collect_to_obj();
    let parent_comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster1", "uuid", &data.parent_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Comment_to_Comment_Cluster1",
            None,
            parent_comment.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct add_story_embedding_Cluster2_batchInput {
    pub story_embeddings: Vec<story_embeddingsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct story_embeddingsData {
    pub story_uuid: String,
    pub content: String,
    pub embedding: Vec<f64>,
}
#[handler]
pub fn add_story_embedding_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    // let data = input.request.in_fmt.deserialize::<add_story_embedding_Cluster2_batchInput>(&input.request.body)?;
    let data = add_story_embedding_Cluster2_batchInput {
        story_embeddings: read_all_chunks::<story_embeddingsData>("story_embeddings")
            .map_err(|e| GraphError::from(e.to_string()))?,
    };
    let mut index = 0;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for story_embeddingsData {
        story_uuid,
        embedding,
        content,
    } in &data.story_embeddings
    {
        let story = G::new(Arc::clone(&db), &txn)
            .n_from_index("Story_Cluster2", "uuid", &story_uuid)
            .collect_to_obj();
        let vector = G::new_mut(Arc::clone(&db), &mut txn)
            .insert_v::<fn(&HVector, &RoTxn) -> bool>(
                &embedding,
                "StoryEmbedding_Cluster2",
                Some(props! { "content" => content.clone() }),
            )
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "Story_to_StoryEmbedding_Cluster2",
                None,
                story.id(),
                vector.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
        index += 1;
        if index % 1000 == 0 {
            println!("Processed {} items", index);
        }
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct drop_all_story_and_embeddings_Cluster2Input {}
#[handler]
pub fn drop_all_story_and_embeddings_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<drop_all_story_and_embeddings_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster2")
            .out("Story_to_StoryEmbedding_Cluster2", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster2")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_triplets_by_subject_uuid_Cluster1Input {
    pub subject_uuid: String,
}
#[handler]
pub fn get_triplets_by_subject_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_triplets_by_subject_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let triplets = G::new(Arc::clone(&db), &txn)
        .n_from_type("Triplet_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("subject_uuid")
                    .map_value_or(false, |v| *v == data.subject_uuid.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct add_comment_embedding_Cluster2Input {
    pub comment_uuid: String,
    pub embedding: Vec<f64>,
    pub content: String,
}
#[handler]
pub fn add_comment_embedding_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<add_comment_embedding_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster2", "uuid", &data.comment_uuid)
        .collect_to_obj();
    let vector = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(
            &data.embedding,
            "CommentEmbedding_Cluster2",
            Some(props! { "content" => data.content.clone() }),
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Comment_to_CommentEmbedding_Cluster2",
            None,
            comment.id(),
            vector.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct add_comment_embedding_Cluster2_batchInput {
    pub comment_embeddings: Vec<comment_embeddingsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct comment_embeddingsData {
    pub content: String,
    pub comment_uuid: String,
    pub embedding: Vec<f64>,
}
#[handler]
pub fn add_comment_embedding_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    // let data = input.request.in_fmt.deserialize::<add_comment_embedding_Cluster2_batchInput>(&input.request.body)?;
    let data = add_comment_embedding_Cluster2_batchInput {
        comment_embeddings: read_all_chunks::<comment_embeddingsData>("comment_embeddings")
            .map_err(|e| GraphError::from(e.to_string()))?,
    };
    println!("Data length: {}", data.comment_embeddings.len());
    let mut index = 0;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = None;
    for comment_embeddingsData {
        comment_uuid,
        embedding,
        content,
    } in &data.comment_embeddings
    {
        if txn.is_none() {
            txn = Some(db.graph_env.write_txn().unwrap());
        }
        let wtxn = txn.as_mut().unwrap();
        let comment = G::new(Arc::clone(&db), wtxn)
            .n_from_index("Comment_Cluster2", "uuid", &comment_uuid)
            .collect_to_obj();
        let vector = G::new_mut(Arc::clone(&db), wtxn)
            .insert_v::<fn(&HVector, &RoTxn) -> bool>(
                &embedding,
                "CommentEmbedding_Cluster2",
                Some(props! { "content" => content.clone() }),
            )
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), wtxn)
            .add_e(
                "Comment_to_CommentEmbedding_Cluster2",
                None,
                comment.id(),
                vector.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
        index += 1;
        if index % 1000 == 0 {
            println!("Processed {} items", index);
        }
        if txn.is_some() && index % 150000 == 0 {
            txn.unwrap().commit().unwrap();
            txn = None;
        }
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.unwrap().commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_comments_Cluster2Input {}
#[handler]
pub fn get_all_comments_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_comments_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let comments = G::new(Arc::clone(&db), &txn)
        .n_from_type("Comment_Cluster2")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comments".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            comments.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct drop_all_tables_Cluster1Input {}
#[handler]
pub fn drop_all_tables_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<drop_all_tables_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Comment_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Chunk_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Event_Cluster1")
            .out("Event_to_Embedding_Cluster1", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Event_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Triplet_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Entity_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct drop_all_Cluster2Input {}
#[handler]
pub fn drop_all_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<drop_all_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("User_Cluster2")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster2")
            .out("Story_to_StoryEmbedding_Cluster2", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster2")
            .out("Story_to_Comment_Cluster2", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Story_Cluster2")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Comment_Cluster2")
            .out("Comment_to_CommentEmbedding_Cluster2", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Comment_Cluster2")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_user_Cluster2Input {
    pub username: String,
}
#[handler]
pub fn insert_user_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_user_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User_Cluster2", Some(props! { "created_at" => chrono::Utc::now().to_rfc3339(), "username" => &data.username }), Some(&["username"])).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_story_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_story_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_story_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "story".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            story.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_chunk_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_chunk_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_chunk_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let chunk = G::new(Arc::clone(&db), &txn)
        .n_from_index("Chunk_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "chunk".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            chunk.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct expire_event_Cluster1Input {
    pub uuid: String,
    pub expired_at: DateTime<Utc>,
}
#[handler]
pub fn expire_event_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<expire_event_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let event = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Event_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "expired_at" => &data.expired_at }))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct drop_all_users_Cluster2Input {}
#[handler]
pub fn drop_all_users_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<drop_all_users_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("User_Cluster2")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_events_without_embeddings_Cluster1Input {}
#[handler]
pub fn get_all_events_without_embeddings_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_events_without_embeddings_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let events = G::new(Arc::clone(&db), &txn)
        .n_from_type("Event_Cluster1")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_comments_by_story_uuid_Cluster2Input {
    pub story_uuid: String,
}
#[handler]
pub fn get_comments_by_story_uuid_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_comments_by_story_uuid_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster2", "uuid", &data.story_uuid)
        .collect_to_obj();
    let comments = G::new_from(Arc::clone(&db), &txn, story.clone())
        .out("Story_to_Comment_Cluster2", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comments".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            comments.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_triplet_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_triplet_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_triplet_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let triplet = G::new(Arc::clone(&db), &txn)
        .n_from_index("Triplet_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplet".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            triplet.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_comment_Cluster1Input {
    pub uuid: String,
    pub username: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}
#[handler]
pub fn insert_comment_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_comment_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster1", Some(props! { "parent_uuid" => &data.parent_uuid, "uuid" => &data.uuid, "username" => &data.username, "text" => &data.text, "created_at" => &data.created_at, "klive" => &data.klive }), Some(&["uuid"])).collect_to_obj();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.parent_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Story_to_Comment_Cluster1",
            None,
            story.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_user_Cluster2_batchInput {
    pub users: Vec<usersData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct usersData {
    pub username: String,
}
#[handler]
pub fn insert_user_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_user_Cluster2_batchInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for usersData { username } in &data.users {
        let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User_Cluster2", Some(props! { "created_at" => chrono::Utc::now().to_rfc3339(), "username" => &username }), Some(&["username"])).collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct connect_user_to_story_Cluster2Input {
    pub username: String,
    pub story_uuid: String,
}
#[handler]
pub fn connect_user_to_story_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<connect_user_to_story_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.username)
        .collect_to_obj();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster2", "uuid", &data.story_uuid)
        .collect_to_obj();
    let edge = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Story_Cluster2",
            None,
            user.id(),
            story.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "edge".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            edge.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_stories_Cluster2Input {}
#[handler]
pub fn get_all_stories_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_stories_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let stories = G::new(Arc::clone(&db), &txn)
        .n_from_type("Story_Cluster2")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct update_triplet_object_Cluster1Input {
    pub uuid: String,
    pub object_uuid: String,
}
#[handler]
pub fn update_triplet_object_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<update_triplet_object_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let triplet = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Triplet_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "object_uuid" => &data.object_uuid }))
            .collect_to_obj()
    };
    let object_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.object_uuid)
        .collect_to_obj();
    Drop::<Vec<_>>::drop_traversal(
        G::new_from(Arc::clone(&db), &txn, triplet.clone())
            .out_e("Triplet_to_Object_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Triplet_to_Object_Cluster1",
            None,
            triplet.id(),
            object_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplet".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            triplet.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_entities_Cluster1Input {}
#[handler]
pub fn get_all_entities_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_entities_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entities = G::new(Arc::clone(&db), &txn)
        .n_from_type("Entity_Cluster1")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_story_Cluster2Input {
    pub uuid: String,
    pub title: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub by: String,
    pub score: i64,
    pub klive: i64,
}
#[handler]
pub fn insert_story_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_story_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let story = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Story_Cluster2", Some(props! { "klive" => &data.klive, "text" => &data.text, "uuid" => &data.uuid, "created_at" => &data.created_at, "url" => &data.url, "score" => &data.score, "title" => &data.title }), Some(&["uuid"])).collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.by)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Story_Cluster2",
            None,
            user.id(),
            story.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "story".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            story.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_event_Cluster1Input {
    pub uuid: String,
    pub chunk_uuid: String,
    pub statement: String,
    pub embedding: Vec<f64>,
    pub triplets: Vec<String>,
    pub statement_type: String,
    pub temporal_type: String,
    pub created_at: DateTime<Utc>,
    pub valid_at: DateTime<Utc>,
}
#[handler]
pub fn insert_event_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_event_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let event = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Event_Cluster1", Some(props! { "temporal_type" => &data.temporal_type, "valid_at" => &data.valid_at, "chunk_uuid" => &data.chunk_uuid, "created_at" => &data.created_at, "statement" => &data.statement, "uuid" => &data.uuid, "triplets" => &data.triplets, "statement_type" => &data.statement_type }), Some(&["uuid"])).collect_to_obj();
    let chunk = G::new(Arc::clone(&db), &txn)
        .n_from_index("Chunk_Cluster1", "uuid", &data.chunk_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Chunk_to_Event_Cluster1",
            None,
            chunk.id(),
            event.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let vector = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(
            &data.embedding,
            "EventEmbedding_Cluster1",
            Some(props! { "embedding" => data.embedding.clone() }),
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Event_to_Embedding_Cluster1",
            None,
            event.id(),
            vector.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_triplet_as_object_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_triplet_as_object_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_triplet_as_object_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let triplets = G::new_from(Arc::clone(&db), &txn, entity.clone())
        .in_("Triplet_to_Object_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_sub_comment_Cluster2_batchInput {
    pub sub_comments: Vec<sub_commentsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct sub_commentsData {
    pub klive: i64,
    pub parent_uuid: String,
    pub uuid: String,
    pub created_at: DateTime<Utc>,
    pub text: String,
    pub username: String,
}
#[handler]
pub fn insert_sub_comment_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_sub_comment_Cluster2_batchInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for sub_commentsData {
        uuid,
        username,
        text,
        created_at,
        klive,
        parent_uuid,
    } in &data.sub_comments
    {
        let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster2", Some(props! { "uuid" => &uuid, "parent_uuid" => &parent_uuid, "text" => &text, "created_at" => &created_at, "klive" => &klive }), Some(&["uuid"])).collect_to_obj();
        let parent_comment = G::new(Arc::clone(&db), &txn)
            .n_from_index("Comment_Cluster2", "uuid", &parent_uuid)
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "Comment_to_Comment_Cluster2",
                None,
                parent_comment.id(),
                comment.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_index("User_Cluster2", "username", &username)
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "User_to_Comments_Cluster2",
                None,
                user.id(),
                comment.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct vector_search_events_Cluster1Input {
    pub query_embedding: Vec<f64>,
    pub k: i32,
}
#[handler]
pub fn vector_search_events_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<vector_search_events_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let matching_embeddings = G::new(Arc::clone(&db), &txn)
        .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
            &data.query_embedding,
            data.k.clone(),
            "EventEmbedding_Cluster1",
            None,
        )
        .collect_to::<Vec<_>>();
    let events = G::new_from(Arc::clone(&db), &txn, matching_embeddings.clone())
        .in_("Event_to_Embedding_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let triplets = G::new_from(Arc::clone(&db), &txn, events.clone())
        .out("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let entities = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .out("Triplet_to_Subject_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let chunks = G::new_from(Arc::clone(&db), &txn, events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_stories_mentioning_entity_as_subject_Cluster1Input {
    pub entity_uuid: String,
}
#[handler]
pub fn get_stories_mentioning_entity_as_subject_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_stories_mentioning_entity_as_subject_Cluster1Input>(
            &input.request.body,
        )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.entity_uuid)
        .collect_to_obj();
    let triplets = G::new_from(Arc::clone(&db), &txn, entity.clone())
        .in_("Triplet_to_Subject_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let events = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .in_("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let chunks = G::new_from(Arc::clone(&db), &txn, events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let stories = G::new_from(Arc::clone(&db), &txn, chunks.clone())
        .in_("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_triplets_Cluster1Input {}
#[handler]
pub fn get_all_triplets_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_triplets_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let triplets = G::new(Arc::clone(&db), &txn)
        .n_from_type("Triplet_Cluster1")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_entity_Cluster1Input {
    pub uuid: String,
    pub event_uuid: String,
    pub name: String,
    pub entity_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}
#[handler]
pub fn insert_entity_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_entity_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let entity = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Entity_Cluster1", Some(props! { "created_at" => &data.created_at, "event_uuid" => &data.event_uuid, "uuid" => &data.uuid, "entity_type" => &data.entity_type, "description" => &data.description, "name" => &data.name }), Some(&["uuid"])).collect_to_obj();
    let event = G::new(Arc::clone(&db), &txn)
        .n_from_index("Event_Cluster1", "uuid", &data.event_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Event_to_Entity_Cluster1",
            None,
            event.id(),
            entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entity".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            entity.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_chunks_by_story_uuid_Cluster1Input {
    pub story_uuid: String,
}
#[handler]
pub fn get_chunks_by_story_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_chunks_by_story_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.story_uuid)
        .collect_to_obj();
    let chunks = G::new_from(Arc::clone(&db), &txn, story.clone())
        .out("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct resolve_entity_Cluster1Input {
    pub uuid: String,
    pub resolved_id: String,
}
#[handler]
pub fn resolve_entity_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<resolve_entity_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let new_entity = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "resolved_id" => &data.resolved_id }))
            .collect_to_obj()
    };
    let old_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.resolved_id)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Resolved_Cluster1",
            None,
            old_entity.id(),
            new_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "new_entity".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            new_entity.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_entities_in_story_Cluster1Input {
    pub story_uuid: String,
}
#[handler]
pub fn get_entities_in_story_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_entities_in_story_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.story_uuid)
        .collect_to_obj();
    let chunks = G::new_from(Arc::clone(&db), &txn, story.clone())
        .out("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let events = G::new_from(Arc::clone(&db), &txn, chunks.clone())
        .out("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let triplets = G::new_from(Arc::clone(&db), &txn, events.clone())
        .out("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let subject_entities = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .out("Triplet_to_Subject_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let object_entities = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .out("Triplet_to_Object_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "subject_entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            subject_entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "object_entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            object_entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_comment_Cluster2Input {
    pub uuid: String,
    pub username: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}
#[handler]
pub fn insert_comment_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_comment_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster2", Some(props! { "created_at" => &data.created_at, "klive" => &data.klive, "text" => &data.text, "uuid" => &data.uuid, "parent_uuid" => &data.parent_uuid }), Some(&["uuid"])).collect_to_obj();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster2", "uuid", &data.parent_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Story_to_Comment_Cluster2",
            None,
            story.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.username)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Comments_Cluster2",
            None,
            user.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_entity_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_entity_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_entity_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entity".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            entity.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct invalidate_event_Cluster1Input {
    pub uuid: String,
    pub invalidated_by: String,
    pub invalid_at: DateTime<Utc>,
}
#[handler]
pub fn invalidate_event_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<invalidate_event_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let event = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Event_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "invalidated_by" => &data.invalidated_by, "invalid_at" => &data.invalid_at }))
    .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_triplet_Cluster1Input {
    pub uuid: String,
    pub event_uuid: String,
    pub subject_name: String,
    pub subject_uuid: String,
    pub predicate: String,
    pub object_name: String,
    pub object_uuid: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}
#[handler]
pub fn insert_triplet_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_triplet_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let triplet = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Triplet_Cluster1", Some(props! { "value" => &data.value, "created_at" => &data.created_at, "object_uuid" => &data.object_uuid, "object_name" => &data.object_name, "subject_name" => &data.subject_name, "uuid" => &data.uuid, "subject_uuid" => &data.subject_uuid, "predicate" => &data.predicate, "event_uuid" => &data.event_uuid }), Some(&["uuid"])).collect_to_obj();
    let event = G::new(Arc::clone(&db), &txn)
        .n_from_index("Event_Cluster1", "uuid", &data.event_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Event_to_Triplet_Cluster1",
            None,
            event.id(),
            triplet.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let subject_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.subject_uuid)
        .collect_to_obj();
    let object_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.object_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Triplet_to_Subject_Cluster1",
            None,
            triplet.id(),
            subject_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Triplet_to_Object_Cluster1",
            None,
            triplet.id(),
            object_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplet".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            triplet.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_triplet_as_subject_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_triplet_as_subject_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_triplet_as_subject_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let triplets = G::new_from(Arc::clone(&db), &txn, entity.clone())
        .in_("Triplet_to_Subject_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct update_event_chunk_Cluster1Input {
    pub uuid: String,
    pub chunk_uuid: String,
}
#[handler]
pub fn update_event_chunk_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<update_event_chunk_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let event = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Event_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "chunk_uuid" => &data.chunk_uuid }))
            .collect_to_obj()
    };
    Drop::<Vec<_>>::drop_traversal(
        G::new_from(Arc::clone(&db), &txn, event.clone())
            .in_e("Chunk_to_Event_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let chunk = G::new(Arc::clone(&db), &txn)
        .n_from_index("Chunk_Cluster1", "uuid", &data.chunk_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Chunk_to_Event_Cluster1",
            None,
            chunk.id(),
            event.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct add_story_embedding_Cluster2Input {
    pub story_uuid: String,
    pub embedding: Vec<f64>,
    pub content: String,
}
#[handler]
pub fn add_story_embedding_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<add_story_embedding_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster2", "uuid", &data.story_uuid)
        .collect_to_obj();
    let vector = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(
            &data.embedding,
            "StoryEmbedding_Cluster2",
            Some(props! { "content" => data.content.clone() }),
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Story_to_StoryEmbedding_Cluster2",
            None,
            story.id(),
            vector.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "story".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            story.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct drop_all_comments_Cluster2Input {
    pub k: i32,
}
#[handler]
pub fn drop_all_comments_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<drop_all_comments_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("Comment_Cluster2")
            .range(0, data.k.clone())
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_stories_Cluster1Input {}
#[handler]
pub fn get_all_stories_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_stories_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let stories = G::new(Arc::clone(&db), &txn)
        .n_from_type("Story_Cluster1")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_sub_comments_by_parent_uuid_Cluster1Input {
    pub parent_uuid: String,
}
#[handler]
pub fn get_sub_comments_by_parent_uuid_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_sub_comments_by_parent_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster1", "uuid", &data.parent_uuid)
        .collect_to_obj();
    let comments = G::new_from(Arc::clone(&db), &txn, comment.clone())
        .out("Comment_to_Comment_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comments".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            comments.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_entity_by_resolved_id_Cluster1Input {
    pub resolved_id: String,
}
#[handler]
pub fn get_entity_by_resolved_id_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_entity_by_resolved_id_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entities = G::new(Arc::clone(&db), &txn)
        .n_from_type("Entity_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("resolved_id")
                    .map_value_or(false, |v| *v == data.resolved_id.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_event_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_event_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_event_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let event = G::new(Arc::clone(&db), &txn)
        .n_from_index("Event_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let embedding = G::new_from(Arc::clone(&db), &txn, event.clone())
        .out("Event_to_Embedding_Cluster1", &EdgeType::Vec)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "embedding".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embedding.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct update_event_Cluster1Input {
    pub uuid: String,
    pub chunk_uuid: String,
    pub statement: String,
    pub embedding: Vec<f64>,
    pub triplets: Vec<String>,
    pub statement_type: String,
    pub temporal_type: String,
    pub created_at: DateTime<Utc>,
    pub valid_at: DateTime<Utc>,
}
#[handler]
pub fn update_event_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<update_event_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let event = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Event_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
    .update(Some(props! { "statement" => &data.statement, "triplets" => &data.triplets, "statement_type" => &data.statement_type, "temporal_type" => &data.temporal_type, "created_at" => &data.created_at, "valid_at" => &data.valid_at }))
    .collect_to_obj()
    };
    Drop::<Vec<_>>::drop_traversal(
        G::new_from(Arc::clone(&db), &txn, event.clone())
            .out("Event_to_Embedding_Cluster1", &EdgeType::Vec)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let vector = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(
            &data.embedding,
            "EventEmbedding_Cluster1",
            Some(props! { "embedding" => data.embedding.clone() }),
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Event_to_Embedding_Cluster1",
            None,
            event.id(),
            vector.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "event".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            event.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct remove_entity_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn remove_entity_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<remove_entity_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
            .collect_to_obj(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_chunk_Cluster1Input {
    pub uuid: String,
    pub story_uuid: String,
    pub text: String,
    pub metadata: String,
}
#[handler]
pub fn insert_chunk_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_chunk_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let chunk = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Chunk_Cluster1", Some(props! { "metadata" => &data.metadata, "text" => &data.text, "story_uuid" => &data.story_uuid, "uuid" => &data.uuid }), Some(&["uuid"])).collect_to_obj();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster1", "uuid", &data.story_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Story_to_Chunk_Cluster1",
            None,
            story.id(),
            chunk.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "chunk".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            chunk.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_stories_by_predicate_Cluster1Input {
    pub predicate: String,
}
#[handler]
pub fn get_stories_by_predicate_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_stories_by_predicate_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let triplets = G::new(Arc::clone(&db), &txn)
        .n_from_type("Triplet_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("predicate")
                    .map_value_or(false, |v| *v == data.predicate.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let events = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .in_("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let chunks = G::new_from(Arc::clone(&db), &txn, events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let stories = G::new_from(Arc::clone(&db), &txn, chunks.clone())
        .in_("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_story_by_uuid_Cluster2Input {
    pub uuid: String,
}
#[handler]
pub fn get_story_by_uuid_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_story_by_uuid_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let story = G::new(Arc::clone(&db), &txn)
        .n_from_index("Story_Cluster2", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "story".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            story.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct connect_user_to_story_Cluster2_batchInput {
    pub user_story_pairs: Vec<user_story_pairsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct user_story_pairsData {
    pub username: String,
    pub story_uuid: String,
}
#[handler]
pub fn connect_user_to_story_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    // let data = input.request.in_fmt.deserialize::<connect_user_to_story_Cluster2_batchInput>(&input.request.body)?;
    let data = connect_user_to_story_Cluster2_batchInput {
        user_story_pairs: read_all_chunks::<user_story_pairsData>("user_story_connections")
            .map_err(|e| GraphError::from(e.to_string()))?,
    };

    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for user_story_pairsData {
        username,
        story_uuid,
    } in &data.user_story_pairs
    {
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_index("User_Cluster2", "username", &username)
            .collect_to_obj();
        let story = G::new(Arc::clone(&db), &txn)
            .n_from_index("Story_Cluster2", "uuid", &story_uuid)
            .collect_to_obj();
        let edge = G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "User_to_Story_Cluster2",
                None,
                user.id(),
                story.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_stories_mentioning_entity_as_object_Cluster1Input {
    pub entity_uuid: String,
}
#[handler]
pub fn get_stories_mentioning_entity_as_object_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_stories_mentioning_entity_as_object_Cluster1Input>(
            &input.request.body,
        )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.entity_uuid)
        .collect_to_obj();
    let triplets = G::new_from(Arc::clone(&db), &txn, entity.clone())
        .in_("Triplet_to_Object_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let events = G::new_from(Arc::clone(&db), &txn, triplets.clone())
        .in_("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let chunks = G::new_from(Arc::clone(&db), &txn, events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let stories = G::new_from(Arc::clone(&db), &txn, chunks.clone())
        .in_("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "chunks".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            chunks.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_user_by_username_Cluster2Input {
    pub username: String,
}
#[handler]
pub fn get_user_by_username_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_user_by_username_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.username)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct connect_user_to_comment_Cluster2Input {
    pub username: String,
    pub comment_uuid: String,
}
#[handler]
pub fn connect_user_to_comment_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<connect_user_to_comment_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.username)
        .collect_to_obj();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster2", "uuid", &data.comment_uuid)
        .collect_to_obj();
    let edge = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Comments_Cluster2",
            None,
            user.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "edge".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            edge.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_story_by_title_Cluster1Input {
    pub title: String,
}
#[handler]
pub fn get_story_by_title_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_story_by_title_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let stories = G::new(Arc::clone(&db), &txn)
        .n_from_type("Story_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("title")
                    .map_value_or(false, |v| *v == data.title.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_comment_by_uuid_Cluster2Input {
    pub uuid: String,
}
#[handler]
pub fn get_comment_by_uuid_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_comment_by_uuid_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster2", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_sub_comments_by_parent_uuid_Cluster2Input {
    pub parent_uuid: String,
}
#[handler]
pub fn get_sub_comments_by_parent_uuid_Cluster2(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_sub_comments_by_parent_uuid_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster2", "uuid", &data.parent_uuid)
        .collect_to_obj();
    let sub_comments = G::new_from(Arc::clone(&db), &txn, comment.clone())
        .out("Comment_to_Comment_Cluster2", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "sub_comments".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            sub_comments.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct has_events_Cluster1Input {}
#[handler]
pub fn has_events_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<has_events_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let events = G::new(Arc::clone(&db), &txn)
        .n_from_type("Event_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("statement_type")
                    .map_value_or(false, |v| *v == "FACT")?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "events".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            events.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct search_entity_with_stories_by_name_Cluster1Input {
    pub entity_name: String,
}
#[handler]
pub fn search_entity_with_stories_by_name_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<search_entity_with_stories_by_name_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let entities = G::new(Arc::clone(&db), &txn)
        .n_from_type("Entity_Cluster1")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("name")
                    .map_value_or(false, |v| *v == data.entity_name.clone())?)
            } else {
                Ok(false)
            }
        })
        .collect_to::<Vec<_>>();
    let subject_triplets = G::new_from(Arc::clone(&db), &txn, entities.clone())
        .in_("Triplet_to_Subject_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let object_triplets = G::new_from(Arc::clone(&db), &txn, entities.clone())
        .in_("Triplet_to_Object_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let subject_events = G::new_from(Arc::clone(&db), &txn, subject_triplets.clone())
        .in_("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let object_events = G::new_from(Arc::clone(&db), &txn, object_triplets.clone())
        .in_("Event_to_Triplet_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let subject_chunks = G::new_from(Arc::clone(&db), &txn, subject_events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let object_chunks = G::new_from(Arc::clone(&db), &txn, object_events.clone())
        .in_("Chunk_to_Event_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let subject_stories = G::new_from(Arc::clone(&db), &txn, subject_chunks.clone())
        .in_("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let object_stories = G::new_from(Arc::clone(&db), &txn, object_chunks.clone())
        .in_("Story_to_Chunk_Cluster1", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entities".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            entities.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "subject_stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            subject_stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "object_stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            object_stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "subject_triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            subject_triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "object_triplets".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            object_triplets.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_all_users_Cluster2Input {}
#[handler]
pub fn get_all_users_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_all_users_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let users = G::new(Arc::clone(&db), &txn)
        .n_from_type("User_Cluster2")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "users".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            users.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_comment_by_uuid_Cluster1Input {
    pub uuid: String,
}
#[handler]
pub fn get_comment_by_uuid_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_comment_by_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster1", "uuid", &data.uuid)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct get_event_embedding_by_event_uuid_Cluster1Input {
    pub event_uuid: String,
}
#[handler]
pub fn get_event_embedding_by_event_uuid_Cluster1(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<get_event_embedding_by_event_uuid_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let event = G::new(Arc::clone(&db), &txn)
        .n_from_index("Event_Cluster1", "uuid", &data.event_uuid)
        .collect_to_obj();
    let embedding = G::new_from(Arc::clone(&db), &txn, event.clone())
        .out("Event_to_Embedding_Cluster1", &EdgeType::Vec)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "embedding".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            embedding.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct connect_user_to_comment_Cluster2_batchInput {
    pub user_comment_pairs: Vec<user_comment_pairsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct user_comment_pairsData {
    pub comment_uuid: String,
    pub username: String,
}
#[handler]
pub fn connect_user_to_comment_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    // let data = input.request.in_fmt.deserialize::<connect_user_to_comment_Cluster2_batchInput>(&input.request.body)?;
    let data = connect_user_to_comment_Cluster2_batchInput {
        user_comment_pairs: read_all_chunks::<user_comment_pairsData>("user_comment_connections")
            .map_err(|e| GraphError::from(e.to_string()))?,
    };
    let mut index = 0;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for user_comment_pairsData {
        username,
        comment_uuid,
    } in &data.user_comment_pairs
    {
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_index("User_Cluster2", "username", &username)
            .collect_to_obj();
        let comment = G::new(Arc::clone(&db), &txn)
            .n_from_index("Comment_Cluster2", "uuid", &comment_uuid)
            .collect_to_obj();
        let edge = G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "User_to_Comments_Cluster2",
                None,
                user.id(),
                comment.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
        index += 1;
        if index % 1000 == 0 {
            println!("Processed {} user comment connections", index);
        }
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_comment_Cluster2_batchInput {
    pub comments: Vec<commentsData>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct commentsData {
    pub created_at: DateTime<Utc>,
    pub uuid: String,
    pub parent_uuid: String,
    pub username: String,
    pub text: String,
    pub klive: i64,
}
#[handler]
pub fn insert_comment_Cluster2_batch(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_comment_Cluster2_batchInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    for commentsData {
        uuid,
        username,
        text,
        created_at,
        klive,
        parent_uuid,
    } in &data.comments
    {
        let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster2", Some(props! { "text" => &text, "parent_uuid" => &parent_uuid, "uuid" => &uuid, "klive" => &klive, "created_at" => &created_at }), Some(&["uuid"])).collect_to_obj();
        let story = G::new(Arc::clone(&db), &txn)
            .n_from_index("Story_Cluster2", "uuid", &parent_uuid)
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "Story_to_Comment_Cluster2",
                None,
                story.id(),
                comment.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
        let user = G::new(Arc::clone(&db), &txn)
            .n_from_index("User_Cluster2", "username", &username)
            .collect_to_obj();
        G::new_mut(Arc::clone(&db), &mut txn)
            .add_e(
                "User_to_Comments_Cluster2",
                None,
                user.id(),
                comment.id(),
                true,
                EdgeType::Node,
            )
            .collect_to_obj();
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct count_all_stories_Cluster2Input {}
#[handler]
pub fn count_all_stories_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<count_all_stories_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let stories = G::new(Arc::clone(&db), &txn)
        .n_from_type("Story_Cluster2")
        .count_to_val();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from(Value::from(stories.clone())),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct update_triplet_subject_Cluster1Input {
    pub uuid: String,
    pub subject_uuid: String,
}
#[handler]
pub fn update_triplet_subject_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<update_triplet_subject_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let triplet = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Triplet_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "subject_uuid" => &data.subject_uuid }))
            .collect_to_obj()
    };
    let subject_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.subject_uuid)
        .collect_to_obj();
    Drop::<Vec<_>>::drop_traversal(
        G::new_from(Arc::clone(&db), &txn, triplet.clone())
            .out_e("Triplet_to_Subject_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Triplet_to_Subject_Cluster1",
            None,
            triplet.id(),
            subject_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "triplet".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            triplet.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct update_entity_resolved_id_Cluster1Input {
    pub uuid: String,
    pub resolved_id: String,
}
#[handler]
pub fn update_entity_resolved_id_Cluster1(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<update_entity_resolved_id_Cluster1Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let entity = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_index("Entity_Cluster1", "uuid", &data.uuid)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "resolved_id" => &data.resolved_id }))
            .collect_to_obj()
    };
    let resolved_entity = G::new(Arc::clone(&db), &txn)
        .n_from_index("Entity_Cluster1", "uuid", &data.resolved_id)
        .collect_to_obj();
    Drop::<Vec<_>>::drop_traversal(
        G::new_from(Arc::clone(&db), &txn, entity.clone())
            .out_e("Resolved_Cluster1")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Resolved_Cluster1",
            None,
            entity.id(),
            resolved_entity.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "entity".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            entity.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct search_similar_stories_Cluster2Input {
    pub query_embedding: Vec<f64>,
    pub k: i64,
}
#[handler]
pub fn search_similar_stories_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<search_similar_stories_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let matching_embeddings = G::new(Arc::clone(&db), &txn)
        .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
            &data.query_embedding,
            data.k.clone(),
            "StoryEmbedding_Cluster2",
            None,
        )
        .collect_to::<Vec<_>>();
    let stories = G::new_from(Arc::clone(&db), &txn, matching_embeddings.clone())
        .in_("Story_to_StoryEmbedding_Cluster2", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "stories".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            stories.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct insert_sub_comment_Cluster2Input {
    pub uuid: String,
    pub username: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub klive: i64,
    pub parent_uuid: String,
}
#[handler]
pub fn insert_sub_comment_Cluster2(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<insert_sub_comment_Cluster2Input>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let comment = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Comment_Cluster2", Some(props! { "uuid" => &data.uuid, "parent_uuid" => &data.parent_uuid, "created_at" => &data.created_at, "klive" => &data.klive, "text" => &data.text }), Some(&["uuid"])).collect_to_obj();
    let parent_comment = G::new(Arc::clone(&db), &txn)
        .n_from_index("Comment_Cluster2", "uuid", &data.parent_uuid)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "Comment_to_Comment_Cluster2",
            None,
            parent_comment.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("User_Cluster2", "username", &data.username)
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "User_to_Comments_Cluster2",
            None,
            user.id(),
            comment.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "comment".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            comment.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

pub fn read_all_chunks<T: serde::de::DeserializeOwned>(
    data_type: &str,
) -> Result<Vec<T>, GraphError> {
    println!("Reading all chunks for {}", data_type);
    let chunk_dir = PathBuf::from("/home/ec2-user/rust/processed_data").join(data_type);
    if !chunk_dir.exists() {
        return Err(GraphError::from("Chunk directory does not exist"));
    }
    println!("Chunk directory exists");
    println!("Reading chunk files");
    let mut all_data = Vec::with_capacity(1000000);
    let mut chunk_files: Vec<_> = fs::read_dir(&chunk_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    println!("Found {} chunk files", chunk_files.len());
    // Sort by filename to process in order
    chunk_files.sort_by_key(|e| e.file_name());
    println!("Sorted chunk files");
    for entry in chunk_files {
        let file = File::open(entry.path())?;
        let reader = std::io::BufReader::new(file);
        let chunk_data: Vec<T> = sonic_rs::from_reader(reader)?;
        all_data.extend(chunk_data);
    }
    println!("Read all chunks");
    println!("Processing {} elements", all_data.len());
    Ok(all_data)
}
