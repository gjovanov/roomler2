use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;
use rand::Rng;
use roomler2_db::models::{
    Conference, ConferenceParticipant, ConferenceSettings, ConferenceStatus, ConferenceType,
    ParticipantRole, ParticipantSession,
};

use super::base::{BaseDao, DaoResult, PaginatedResult, PaginationParams};

pub struct ConferenceDao {
    pub base: BaseDao<Conference>,
    pub participants: BaseDao<ConferenceParticipant>,
}

impl ConferenceDao {
    pub fn new(db: &Database) -> Self {
        Self {
            base: BaseDao::new(db, Conference::COLLECTION),
            participants: BaseDao::new(db, ConferenceParticipant::COLLECTION),
        }
    }

    pub async fn create(
        &self,
        tenant_id: ObjectId,
        organizer_id: ObjectId,
        subject: String,
        conference_type: ConferenceType,
        channel_id: Option<ObjectId>,
    ) -> DaoResult<Conference> {
        let meeting_code = generate_meeting_code();
        let join_url = format!("/join/{}", meeting_code);
        let now = DateTime::now();

        let conference = Conference {
            id: None,
            tenant_id,
            channel_id,
            subject,
            description: None,
            conference_type,
            status: ConferenceStatus::Scheduled,
            start_time: None,
            end_time: None,
            actual_start_time: None,
            actual_end_time: None,
            duration: None,
            timezone: None,
            recurrence: None,
            join_url,
            meeting_code,
            passcode: None,
            waiting_room: false,
            organizer_id,
            co_organizer_ids: Vec::new(),
            settings: ConferenceSettings::default(),
            participant_count: 0,
            peak_participant_count: 0,
            created_at: now,
            updated_at: now,
        };

        let id = self.base.insert_one(&conference).await?;
        self.base.find_by_id(id).await
    }

    pub async fn start(&self, conference_id: ObjectId) -> DaoResult<bool> {
        self.base
            .update_by_id(
                conference_id,
                doc! {
                    "$set": {
                        "status": "in_progress",
                        "actual_start_time": DateTime::now(),
                    }
                },
            )
            .await
    }

    pub async fn end(&self, conference_id: ObjectId) -> DaoResult<bool> {
        self.base
            .update_by_id(
                conference_id,
                doc! {
                    "$set": {
                        "status": "ended",
                        "actual_end_time": DateTime::now(),
                    }
                },
            )
            .await
    }

    pub async fn join_participant(
        &self,
        tenant_id: ObjectId,
        conference_id: ObjectId,
        user_id: ObjectId,
        display_name: String,
        device_type: String,
    ) -> DaoResult<ConferenceParticipant> {
        let now = DateTime::now();
        let session = ParticipantSession {
            joined_at: now,
            left_at: None,
            duration: None,
            device_type,
        };

        let participant = ConferenceParticipant {
            id: None,
            tenant_id,
            conference_id,
            user_id: Some(user_id),
            display_name,
            email: None,
            is_external: false,
            role: ParticipantRole::Attendee,
            sessions: vec![session],
            is_muted: false,
            is_video_on: true,
            is_screen_sharing: false,
            is_hand_raised: false,
            total_duration: 0,
            created_at: now,
            updated_at: now,
        };

        let id = self.participants.insert_one(&participant).await?;

        // Increment participant count
        self.base
            .update_by_id(
                conference_id,
                doc! { "$inc": { "participant_count": 1 } },
            )
            .await?;

        self.participants.find_by_id(id).await
    }

    pub async fn leave_participant(
        &self,
        conference_id: ObjectId,
        user_id: ObjectId,
    ) -> DaoResult<bool> {
        let now = DateTime::now();
        // Update the last session's left_at
        let filter = doc! {
            "conference_id": conference_id,
            "user_id": user_id,
        };
        let update = doc! {
            "$set": {
                "sessions.$[elem].left_at": now,
                "updated_at": now,
            }
        };
        // Use raw collection to apply array filters
        let opts = mongodb::options::UpdateOptions::builder()
            .array_filters(vec![doc! { "elem.left_at": null }])
            .build();
        self.participants
            .collection()
            .update_one(filter, update)
            .with_options(opts)
            .await
            .map_err(|e| super::base::DaoError::Mongo(e))?;

        // Decrement participant count
        self.base
            .update_by_id(
                conference_id,
                doc! { "$inc": { "participant_count": -1 } },
            )
            .await?;

        Ok(true)
    }

    pub async fn list_participants(
        &self,
        conference_id: ObjectId,
    ) -> DaoResult<Vec<ConferenceParticipant>> {
        self.participants
            .find_many(doc! { "conference_id": conference_id }, Some(doc! { "created_at": 1 }))
            .await
    }

    pub async fn list_by_tenant(
        &self,
        tenant_id: ObjectId,
        params: &PaginationParams,
    ) -> DaoResult<PaginatedResult<Conference>> {
        self.base
            .find_paginated(
                doc! { "tenant_id": tenant_id },
                Some(doc! { "created_at": -1 }),
                params,
            )
            .await
    }
}

fn generate_meeting_code() -> String {
    let mut rng = rand::rng();
    let parts: Vec<String> = (0..3)
        .map(|_| {
            let n: u32 = rng.random_range(100..999);
            n.to_string()
        })
        .collect();
    parts.join("-")
}
