use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref GET_USER: &'static str = r"
        SELECT id,email,display_name,phone,password,created_at,admin,enabled
            FROM users
            WHERE email = ?1
    ";

    pub(crate) static ref CREATE_USER: &'static str = r"
        INSERT INTO users(id,email,password,display_name) VALUES (?, ?, ?, ?) RETURNING *
    ";

    pub(crate) static ref GET_ALL_JOBS: &'static str = r"SELECT * FROM jobs";

    pub(crate) static ref CREATE_JOB: &'static str = r"INSERT INTO jobs(id,synopsis,location,created_by) VALUES (?, ?, ?, ?) RETURNING *";

    pub(crate) static ref GET_JOB_BY_ID: &'static str = r"SELECT * FROM jobs WHERE id = ?";

    pub(crate) static ref GET_COMMENTS_FOR_JOB: &'static str = r"SELECT * FROM comments WHERE job_id = ?";

    pub(crate) static ref ADD_COMMENT: &'static str = r"INSERT INTO comments(id,job_id,comment,created_by) VALUES (?, ?, ?, ?) RETURNING *";
}
