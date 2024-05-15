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
}
