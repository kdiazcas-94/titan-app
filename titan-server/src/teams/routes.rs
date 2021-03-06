use rocket::{get, State, http::RawStr};
use rocket::http::Status;
use rocket::request::{Form};
use rocket_contrib::json::Json;
use serde::Deserialize;

use crate::db::{UnksoMainForums, TitanPrimary};
use crate::models;
use crate::config;
use crate::teams;
use crate::accounts;
use crate::guards::form::NaiveDateTimeForm;
use crate::accounts::file_entries;
use crate::guards::auth_guard;
use crate::teams::roles::RoleRankScope;
use crate::api::{ApiResponse, ApiError};

#[get("/")]
pub fn get_all(titan_primary: TitanPrimary) -> ApiResponse<Vec<models::Organization>> {
    ApiResponse::from(teams::organizations::find_all(titan_primary))
}

#[get("/<id>", rank = 1)]
pub fn get_organization_by_id(
    id: i32,
    titan_db: TitanPrimary
) -> ApiResponse<models::Organization> {
    ApiResponse::from(teams::organizations::find_by_id(id, &*titan_db))
}

#[get("/<slug>", rank = 2)]
pub fn get_organization_by_slug(
    slug: &RawStr,
    unkso_titan: TitanPrimary
) -> ApiResponse<models::Organization> {
    ApiResponse::from(teams::organizations::find_by_slug(slug.as_str(), &unkso_titan))
}

#[get("/<org_id>/children")]
pub fn get_child_organizations(
    org_id: i32,
    titan_db: TitanPrimary
) -> ApiResponse<Vec<models::Organization>> {
    ApiResponse::from(teams::organizations::find_children(org_id, &*titan_db))
}

/** ******************************************************************
 *  Roles/members
 ** *****************************************************************/
#[get("/<id>/users?<children>")]
pub fn get_organization_users(
    id: i32,
    children: Option<bool>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<Vec<models::UserProfile>> {
    let include_children = children.unwrap_or(false);
    let users = teams::organizations::find_users(id, include_children, &*titan_db)
        .and_then(|users| accounts::users::map_users_to_profile(&users, &*wcf_db, &app_config));
    ApiResponse::from(users)
}

#[derive(Deserialize)]
pub struct AddUserRequest {
    #[serde(rename(serialize = "userId", deserialize = "userId"))]
    user_id: i32,
}

/// TODO Verify CoC permissions.
#[post("/<org_id>/users", format = "application/json", data = "<user_fields>")]
pub fn add_user(
    org_id: i32,
    user_fields: Json<AddUserRequest>,
    titan_db: TitanPrimary,
) -> Json<bool> {
    let titan_db_ref = &*titan_db;
    let org_user = &models::OrganizationUser {
        organization_id: org_id,
        user_id: user_fields.user_id,
    };

    if teams::organizations::is_user_org_member(org_user, titan_db_ref) {
        return Json(true);
    }

    let res = teams::organizations::add_user(
        &org_user, titan_db_ref);
    Json(res.is_ok())
}

#[derive(Deserialize)]
pub struct RemoveUserRequest {
    #[serde(rename(serialize = "userId", deserialize = "userId"))]
    user_id: i32,
}

/// TODO Verify CoC permissions.
#[delete("/<org_id>/users", format = "application/json", data = "<user_fields>")]
pub fn remove_user(
    org_id: i32,
    user_fields: Json<RemoveUserRequest>,
    titan_db: TitanPrimary,
) -> ApiResponse<bool> {
    let res = teams::organizations::remove_user(&models::OrganizationUser {
        organization_id: org_id,
        user_id: user_fields.user_id,
    }, &*titan_db);
    ApiResponse::from(res.is_ok())
}

#[get("/<org_id>/users/<user_id>/coc")]
pub fn get_organization_user_coc(
    org_id: i32,
    user_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<models::ChainOfCommand> {
    ApiResponse::from(teams::roles::find_user_coc(
        org_id, user_id, &*titan_db, &*wcf_db, app_config))
}

#[get("/<org_id>/coc")]
pub fn get_organization_coc(
    org_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<models::ChainOfCommand> {
    ApiResponse::from(teams::roles::find_org_coc(
        org_id, std::i32::MAX, &*titan_db, &*wcf_db, &app_config))
}

#[get("/<org_id>/roles?<scope>")]
pub fn list_organization_roles(
    org_id: i32,
    scope: RoleRankScope,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    _auth_user: auth_guard::AuthenticatedUser
) -> ApiResponse<Vec<models::OrganizationRoleWithAssoc>> {
    let roles = teams::roles::find_org_roles(org_id, scope, &*titan_db)
        .and_then(|roles| teams::roles::map_roles_assoc(roles, &*titan_db, &*wcf_db, &app_config));
    ApiResponse::from(roles)
}

/// [deprecated(note = "Use get_organization_roles instead.")]
#[get("/<org_id>/roles/unranked")]
pub fn get_organization_unranked_roles(
    org_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<Vec<models::OrganizationRoleWithAssoc>> {
    ApiResponse::from(teams::roles::find_unranked_roles(
        org_id, &*titan_db, &*wcf_db, &app_config))
}

#[get("/roles/<role_id>/parent")]
pub fn get_parent_role(
    role_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<Option<models::OrganizationRoleWithAssoc>> {
    let titan_db_ref = &*titan_db;
    let res = teams::roles::find_parent_role(role_id, false, titan_db_ref)
        .and_then(|parent_role| {
            match parent_role {
                Some(role) =>  teams::roles::map_role_assoc(
                    &role, titan_db_ref, &*wcf_db, &app_config)
                    .map(Some),
                None => Ok(None),
            }
        });
    ApiResponse::from(res)
}

#[derive(Deserialize)]
pub struct ReorderRolesRequest {
    #[serde(alias = "roleIds")]
    role_ids: Vec<i32>,
}

#[post("/<org_id>/roles:reorder", format = "application/json", data = "<request>")]
pub fn reorder_roles(
    org_id: i32,
    request: Json<ReorderRolesRequest>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    auth_user: auth_guard::AuthenticatedUser
) -> ApiResponse<Option<String>> {
    let titan_db_ref = &*titan_db;
    let is_authorized = teams::roles::is_user_in_parent_coc(
        auth_user.user.id, org_id, titan_db_ref, &*wcf_db, &app_config);

    if !is_authorized {
        return ApiResponse::from(ApiError::ValidationError("User not found in parent CoC."));
    }

    ApiResponse::from(
        teams::roles::reorder_roles(org_id, &request.role_ids, &*titan_db)
            .map(|_| None))
}

#[post("/<org_id>/roles", format = "application/json", data = "<fields>")]
pub fn create_organization_role(
    org_id: i32,
    fields: Json<models::UpdateOrganizationRole>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    _auth_user: auth_guard::AuthenticatedUser,
) -> ApiResponse<models::OrganizationRoleWithAssoc> {
    let titan_db_ref = &*titan_db;
    let wcf_db_ref = &*wcf_db;
    let is_valid = match fields.user_id {
        Some(user_id) => !teams::roles::is_user_in_coc(
            user_id, org_id, titan_db_ref, wcf_db_ref, &app_config),
        None => true,
    };

    if !is_valid {
        return ApiResponse::from(ApiError::ValidationError("Assignee is already in CoC"));
    }

    let role = teams::roles::create_role(&models::NewOrganizationRole {
        organization_id: org_id,
        user_id: fields.user_id,
        role: fields.role.clone(),
        rank: fields.rank,
    }, titan_db_ref)
        .and_then(|role| teams::roles::map_role_assoc(
            &role, titan_db_ref, wcf_db_ref, &app_config));

    ApiResponse::from(role)
}

#[post("/<org_id>/roles/<role_id>", format = "application/json", data = "<fields>")]
pub fn update_organization_role(
    role_id: i32,
    org_id: i32,
    fields: Json<models::UpdateOrganizationRole>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<models::OrganizationRoleWithAssoc> {
    let titan_db_ref = &*titan_db;
    let wcf_db_ref = &*wcf_db;
    let update = models::UpdateOrganizationRole {
        user_id: fields.user_id,
        role: fields.role.clone(),
        rank: fields.rank,
    };
    let is_in_coc = match fields.user_id {
        Some(user_id) => {
            // Checks if the user is assigned to another role
            // in the same COC.
            teams::roles::find_role_in_coc(user_id, org_id, titan_db_ref, wcf_db_ref, &app_config)
                .map_or(false, |role| role.organization.id != org_id)
        },
        None => false,
    };

    if is_in_coc {
        return ApiResponse::from(ApiError::ValidationError("Assignee is already in CoC"));
    }

    let role = teams::roles::update_role(role_id, org_id, &update, titan_db_ref)
        .and_then(|role| teams::roles::map_role_assoc(
            &role, titan_db_ref, wcf_db_ref, &app_config));

    ApiResponse::from(role)
}

/** ******************************************************************
 *  Reports
 ** *****************************************************************/
#[get("/<org_id>/reports")]
pub fn list_organization_reports(
    org_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    auth_user: auth_guard::AuthenticatedUser
) -> ApiResponse<Vec<models::ReportWithAssoc>> {
    let titan_db_ref = &*titan_db;
    let wcf_db_ref = &*wcf_db;
    let role_res = teams::roles::find_role_in_coc(
        auth_user.user.id, org_id, titan_db_ref, wcf_db_ref, &app_config);
    let res = match role_res {
        Some(role) => {
            if role.organization.id == org_id {
                // `role.rank` is safe to unwrap because `find_role_in_coc()` filters
                // out non-ranked roles. Therefore, it will always have a value.
                teams::reports::find_all_by_org_up_to_rank(
                    org_id, role.rank.unwrap(), titan_db_ref, &*wcf_db, &app_config)
                    .map_err(ApiError::from)
            } else {
                teams::reports::find_all_by_organization(
                    org_id, titan_db_ref, &*wcf_db, &app_config)
                    .map_err(ApiError::from)
            }
        }
        None => Err(ApiError::AuthenticationError)
    };
    ApiResponse::from(res)
}

#[get("/reports/unacknowledged", format = "application/json")]
pub fn get_all_unacknowledged_reports(
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    auth_user: auth_guard::AuthenticatedUser
) -> ApiResponse<Vec<models::ReportWithAssoc>> {
    let titan_db_ref = &*titan_db;
    let wcf_db_ref = &*wcf_db;
    let res = teams::roles::find_ranked_by_user_id(auth_user.user.id, titan_db_ref)
        .and_then(|mut roles| {
            let mut direct_report_ids: Vec<i32> = vec!();
            for role in roles.drain(..) {
                let direct_reports_res = teams::roles::find_direct_reports(
                    role.organization_id, role.rank.unwrap(), titan_db_ref, wcf_db_ref, &app_config);
                match direct_reports_res {
                    Ok(mut direct_reports) => {
                        direct_report_ids.append(&mut direct_reports.iter_mut()
                            .map(|role| role.id).collect());
                    }
                    Err(err) => {
                        return Err(err)
                    }
                }
            }
            Ok(direct_report_ids)
        })
        .and_then(|direct_report_ids| teams::reports::find_unacknowledged_by_role_ids(
            &direct_report_ids, titan_db_ref, wcf_db_ref, &app_config))
        .map_err(ApiError::from);
    ApiResponse::from(res)
}

#[derive(Deserialize)]
pub struct CreateOrganizationReportRequest {
    comments: String,
    term_start_date: chrono::NaiveDateTime,
}

#[post("/<org_id>/reports", format = "application/json", data = "<report_form>")]
pub fn create_organization_report(
    org_id: i32,
    report_form: Json<CreateOrganizationReportRequest>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    auth_user: auth_guard::AuthenticatedUser,
) -> ApiResponse<models::ReportWithAssoc> {
    let titan_db_ref = &*titan_db;
    let res = teams::roles::find_org_role_by_user_id(org_id, auth_user.user.id, titan_db_ref)
        .map(|role| models::NewReport {
            role_id: role.id,
            term_start_date: report_form.term_start_date,
            submission_date: Some(chrono::Utc::now().naive_utc()),
            comments: Some(report_form.comments.clone()),
            ack_user_id: None,
            ack_date: None,
            date_created: chrono::Utc::now().naive_utc(),
            date_modified: chrono::Utc::now().naive_utc(),
        })
        .and_then(|new_report| teams::reports::save_report(
            &new_report, titan_db_ref, &*wcf_db, &app_config));
    ApiResponse::from(res)
}

#[post("/<org_id>/reports/<report_id>/ack")]
pub fn ack_organization_report(
    org_id: i32,
    report_id: i32,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>,
    auth_user: auth_guard::AuthenticatedUser
) -> ApiResponse<models::ReportWithAssoc> {
    let titan_db_ref = &*titan_db;
    let res = teams::reports::find_by_id(report_id, titan_db_ref)
        // Ensure the authenticated user holds the role that is directly
        // above the role that submitted the report.
        .and_then(|report| teams::roles::find_parent_role(
            report.role_id, false, titan_db_ref))
        .map_err(ApiError::from)
        .and_then(|role_option| role_option
            .map(|role| {
                if role.user_id.is_none() || role.user_id.unwrap() != auth_user.user.id {
                    return Err(ApiError::AuthenticationError)
                }
                Ok(role)
            })
            .unwrap_or_else(|| Err(ApiError::AuthenticationError)))
        // Acknowledge and return the report
        .and_then(|_| teams::reports::ack_report(
            report_id, auth_user.user.id, titan_db_ref).map_err(ApiError::from))
        .and_then(|report| teams::reports::map_report_to_assoc(
            report, titan_db_ref, &*wcf_db, &app_config).map_err(ApiError::from));
    ApiResponse::from(res)
}

/** ******************************************************************
 *  File entries
 ** *****************************************************************/
#[derive(FromForm)]
pub struct ListOrgUserFileEntriesRequest {
    /// A string delimited list of organization Ids.
    pub organizations: String,
    pub from_start_date: NaiveDateTimeForm,
    pub to_start_date: NaiveDateTimeForm,
}

#[get("/file-entries?<fields..>")]
pub fn list_organization_user_file_entries(
    fields: Form<ListOrgUserFileEntriesRequest>,
    titan_db: TitanPrimary,
    wcf_db: UnksoMainForums,
    app_config: State<config::AppConfig>
) -> ApiResponse<Vec<models::UserFileEntryWithAssoc>> {
    let ListOrgUserFileEntriesRequest {
        organizations,
        from_start_date,
        to_start_date
    } = fields.into_inner();
    let org_ids = organizations.split(',')
        .map(|id| id.parse::<i32>().unwrap())
        .collect();
    ApiResponse::from(file_entries::find_by_orgs(
        org_ids,
        *from_start_date,
        *to_start_date,
        &*titan_db,
        &*wcf_db,
        &app_config
    ))
}
