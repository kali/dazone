
#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Ranking {
    pub url: String,
    pub pagerank: u64,
    pub duration: u64,
}

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct UserVisits {
    pub source_ip: String,
    pub dest_url: String,
    pub visit_date: String,
    pub ad_revenue: f32,
    pub user_agent: String,
    pub country_code: String,
    pub language_code: String,
    pub search_word: String,
    pub duration: u32,
}
