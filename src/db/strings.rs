use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref GET_USER_BY_EMAIL: &'static str = r"
        SELECT id,email,display_name,phone,password,created_at,admin,enabled
            FROM users
            WHERE email = ?1
    ";
    pub(crate) static ref GET_USER_BY_ID: &'static str = r"SELECT * FROM users WHERE id = ?1";
    pub(crate) static ref CREATE_USER: &'static str = r"
        INSERT INTO users(id,email,password,display_name) VALUES (?, ?, ?, ?) RETURNING *
    ";
    pub(crate) static ref GET_ALL_JOBS: &'static str = r"SELECT * FROM jobs";
    pub(crate) static ref CREATE_JOB: &'static str = r"INSERT INTO jobs(id,synopsis,location,caller_name,caller_phone,created_by) VALUES (?, ?, ?, ?, ?, ?) RETURNING *";
    pub(crate) static ref GET_JOB_BY_ID: &'static str = r"SELECT * FROM jobs WHERE id = ?";
    pub(crate) static ref CLOSE_JOB: &'static str =
        r"UPDATE jobs SET closed_at = (strftime('%s','now')), closed_by = ? WHERE id = ?";
    pub(crate) static ref GET_COMMENTS_FOR_JOB: &'static str =
        r"SELECT * FROM comments WHERE job_id = ?";
    pub(crate) static ref ADD_COMMENT: &'static str =
        r"INSERT INTO comments(id,job_id,comment,created_by) VALUES (?, ?, ?, ?) RETURNING *";
    pub(crate) static ref GET_ACTIVE_ASSIGNMENTS: &'static str = r"SELECT * FROM assignments WHERE removed_at IS NULL AND job_id IN (SELECT id FROM jobs WHERE closed_at IS NULL);";
    pub(crate) static ref GET_ASSIGNMENTS_BY_JOBID: &'static str =
        r"SELECT * FROM assignments WHERE job_id = ?";
    pub(crate) static ref CREATE_ASSIGNMENT: &'static str = r"INSERT INTO assignments(id,job_id,resource_id,assigned_by) VALUES (?, ?, ?, ?) RETURNING *";
    pub(crate) static ref REMOVE_ASSIGNMENT: &'static str =
        r"UPDATE assignments SET removed_at = (strftime('%s','now')), removed_by = ? WHERE id = ?";
    pub(crate) static ref CLOSE_ASSIGNMENTS_FOR_JOB: &'static str = r"UPDATE assignments SET removed_at = (strftime('%s','now')), removed_by = ? WHERE job_id = ? AND removed_at IS NULL";
    pub(crate) static ref CREATE_RESOURCE: &'static str =
        r"INSERT INTO resources(id,display_name,comment) VALUES (?, ?, ?) RETURNING *";
    pub(crate) static ref UPDATE_RESOURCE_IN_SERVICE: &'static str =
        r"UPDATE resources SET in_service = ? WHERE id = ?";
    pub(crate) static ref UPDATE_ASSIGNMENTS_RESOURCE_OOS: &'static str = r"UPDATE assignments
            SET removed_at = (strftime('%s','now')), removed_by = ?
            WHERE resource_id = ? AND removed_at IS NULL";
    pub(crate) static ref GET_RESOURCES: &'static str = r"
        WITH aa AS (
            SELECT id,job_id,resource_id,assigned_at,removed_at,assigned_by,removed_by
                FROM assignments
                WHERE removed_at IS NULL
                    AND job_id IN (SELECT id FROM jobs WHERE closed_at IS NULL))
        SELECT resources.id as resource_id,resources.display_name,resources.in_service,resources.comment,aa.id as aa_id,aa.job_id,aa.assigned_at,aa.assigned_by,aa.removed_at,aa.removed_by FROM resources
            LEFT OUTER JOIN aa
                ON resources.id = aa.resource_id;";
}
