// This file was generated with `clorinde`. Do not modify.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum SearchStatus {
    Active,
    Open,
    Closed,
}
impl<'a> postgres_types::ToSql for SearchStatus {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            SearchStatus::Active => "active",
            SearchStatus::Open => "open",
            SearchStatus::Closed => "closed",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "search_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "active" => true,
                    "open" => true,
                    "closed" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for SearchStatus {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<SearchStatus, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "active" => Ok(SearchStatus::Active),
            "open" => Ok(SearchStatus::Open),
            "closed" => Ok(SearchStatus::Closed),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "search_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "active" => true,
                    "open" => true,
                    "closed" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum UserRole {
    Candidate,
    Employer,
    Admin,
}
impl<'a> postgres_types::ToSql for UserRole {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            UserRole::Candidate => "candidate",
            UserRole::Employer => "employer",
            UserRole::Admin => "admin",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "user_role" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "candidate" => true,
                    "employer" => true,
                    "admin" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for UserRole {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<UserRole, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "candidate" => Ok(UserRole::Candidate),
            "employer" => Ok(UserRole::Employer),
            "admin" => Ok(UserRole::Admin),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "user_role" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "candidate" => true,
                    "employer" => true,
                    "admin" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum InvitationStatus {
    Sent,
    Viewed,
    Accepted,
    Declined,
    Expired,
    Withdrawn,
}
impl<'a> postgres_types::ToSql for InvitationStatus {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            InvitationStatus::Sent => "sent",
            InvitationStatus::Viewed => "viewed",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Declined => "declined",
            InvitationStatus::Expired => "expired",
            InvitationStatus::Withdrawn => "withdrawn",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "invitation_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 6 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "sent" => true,
                    "viewed" => true,
                    "accepted" => true,
                    "declined" => true,
                    "expired" => true,
                    "withdrawn" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for InvitationStatus {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<InvitationStatus, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "sent" => Ok(InvitationStatus::Sent),
            "viewed" => Ok(InvitationStatus::Viewed),
            "accepted" => Ok(InvitationStatus::Accepted),
            "declined" => Ok(InvitationStatus::Declined),
            "expired" => Ok(InvitationStatus::Expired),
            "withdrawn" => Ok(InvitationStatus::Withdrawn),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "invitation_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 6 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "sent" => true,
                    "viewed" => true,
                    "accepted" => true,
                    "declined" => true,
                    "expired" => true,
                    "withdrawn" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum TaskAssignmentStatus {
    Assigned,
    Submitted,
    Reviewed,
    Skipped,
    Expired,
}
impl<'a> postgres_types::ToSql for TaskAssignmentStatus {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            TaskAssignmentStatus::Assigned => "assigned",
            TaskAssignmentStatus::Submitted => "submitted",
            TaskAssignmentStatus::Reviewed => "reviewed",
            TaskAssignmentStatus::Skipped => "skipped",
            TaskAssignmentStatus::Expired => "expired",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "task_assignment_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 5 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "assigned" => true,
                    "submitted" => true,
                    "reviewed" => true,
                    "skipped" => true,
                    "expired" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for TaskAssignmentStatus {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<TaskAssignmentStatus, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "assigned" => Ok(TaskAssignmentStatus::Assigned),
            "submitted" => Ok(TaskAssignmentStatus::Submitted),
            "reviewed" => Ok(TaskAssignmentStatus::Reviewed),
            "skipped" => Ok(TaskAssignmentStatus::Skipped),
            "expired" => Ok(TaskAssignmentStatus::Expired),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "task_assignment_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 5 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "assigned" => true,
                    "submitted" => true,
                    "reviewed" => true,
                    "skipped" => true,
                    "expired" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum VacancyStatus {
    Draft,
    Published,
    Closed,
}
impl<'a> postgres_types::ToSql for VacancyStatus {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            VacancyStatus::Draft => "draft",
            VacancyStatus::Published => "published",
            VacancyStatus::Closed => "closed",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "vacancy_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "draft" => true,
                    "published" => true,
                    "closed" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for VacancyStatus {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<VacancyStatus, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "draft" => Ok(VacancyStatus::Draft),
            "published" => Ok(VacancyStatus::Published),
            "closed" => Ok(VacancyStatus::Closed),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "vacancy_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 3 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "draft" => true,
                    "published" => true,
                    "closed" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[allow(non_camel_case_types)]
pub enum ApplicationStatus {
    Sent,
    Viewed,
    Accepted,
    Rejected,
    Withdrawn,
}
impl<'a> postgres_types::ToSql for ApplicationStatus {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        buf: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = match *self {
            ApplicationStatus::Sent => "sent",
            ApplicationStatus::Viewed => "viewed",
            ApplicationStatus::Accepted => "accepted",
            ApplicationStatus::Rejected => "rejected",
            ApplicationStatus::Withdrawn => "withdrawn",
        };
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "application_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 5 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "sent" => true,
                    "viewed" => true,
                    "accepted" => true,
                    "rejected" => true,
                    "withdrawn" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}
impl<'a> postgres_types::FromSql<'a> for ApplicationStatus {
    fn from_sql(
        ty: &postgres_types::Type,
        buf: &'a [u8],
    ) -> Result<ApplicationStatus, Box<dyn std::error::Error + Sync + Send>> {
        match std::str::from_utf8(buf)? {
            "sent" => Ok(ApplicationStatus::Sent),
            "viewed" => Ok(ApplicationStatus::Viewed),
            "accepted" => Ok(ApplicationStatus::Accepted),
            "rejected" => Ok(ApplicationStatus::Rejected),
            "withdrawn" => Ok(ApplicationStatus::Withdrawn),
            s => Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        if ty.name() != "application_status" {
            return false;
        }
        match *ty.kind() {
            postgres_types::Kind::Enum(ref variants) => {
                if variants.len() != 5 {
                    return false;
                }
                variants.iter().all(|v| match &**v {
                    "sent" => true,
                    "viewed" => true,
                    "accepted" => true,
                    "rejected" => true,
                    "withdrawn" => true,
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
