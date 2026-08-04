use mysql::*;
fn test() {
    let opts = Opts::from_url("mysql://root@localhost/db").unwrap();
    let pool = Pool::new(opts).unwrap();
    let mut conn = pool.get_conn().unwrap();
}
