use std::io::Write;

use cap::{ranking, user_visits};
use capnp::message::Builder;
use capnp::serialize_packed;

use Dx16Result;

pub trait Capitanable {
    fn write_to_cap<W: Write>(&self, w: &mut W) -> Dx16Result<()>;
}

#[derive(RustcDecodable,RustcEncodable,Debug)]
pub struct Ranking {
    pub url: String,
    pub pagerank: u64,
    pub duration: u64,
}

impl Capitanable for Ranking {
    fn write_to_cap<W: Write>(&self, w: &mut W) -> Dx16Result<()> {
        let mut message = Builder::new_default();
        {
            let mut ranking = message.init_root::<ranking::Builder>();
            ranking.set_url(&*self.url);
            ranking.set_pagerank(self.pagerank);
            ranking.set_duration(self.duration);
        }
        try!(serialize_packed::write_message(w, &mut message));
        Ok(())
    }
}

// "CREATE EXTERNAL TABLE uservisits (sourceIP STRING,destURL STRING," \
// "visitDate STRING,adRevenue DOUBLE,userAgent STRING,countryCode STRING," \
// "languageCode STRING,searchWord STRING,duration INT ) " \
// "ROW FORMAT DELIMITED FIELDS TERMINATED BY \\\",\\\" " \

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

impl Capitanable for UserVisits {
    fn write_to_cap<W: Write>(&self, w: &mut W) -> Dx16Result<()> {
        let mut message = Builder::new_default();
        {
            let mut it = message.init_root::<user_visits::Builder>();
            it.set_source_i_p(&*self.source_ip);
            it.set_dest_u_r_l(&*self.dest_url);
            it.set_visit_date(&*self.visit_date);
            it.set_ad_revenue(self.ad_revenue);
            it.set_user_agent(&*self.user_agent);
            it.set_country_code(&*self.country_code);
            it.set_language_code(&*self.language_code);
            it.set_search_word(&*self.search_word);
            it.set_duration(self.duration as u64);
        }
        try!(serialize_packed::write_message(w, &mut message));
        Ok(())
    }
}
