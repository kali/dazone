use std::io::Write;

use capdata::{ranking, user_visits};
use capnp::message::Builder;
use capnp::serialize_packed;
use capnp::serialize;
use capnp::message::Allocator;

use byteorder::{WriteBytesExt, LittleEndian};

use data::pod::{UserVisits, Ranking};

use Dx16Result;

#[derive(PartialEq,Clone,Copy)]
pub enum Mode {
    Packed, Unpacked, Mappable
}

pub trait Capitanable {
    fn write_to_cap<W: Write>(&self, w: &mut W, mode:Mode) -> Dx16Result<()>;
}

fn write_message<W:Write,A:Allocator>(message:&Builder<A>, w: &mut W, mode:Mode) -> Dx16Result<()> {
    Ok(try!(match mode {
        Mode::Packed => serialize_packed::write_message(w, message),
        Mode::Unpacked => serialize::write_message(w, message),
        Mode::Mappable => {
            w.write_u64::<LittleEndian>(serialize::compute_serialized_size_in_words(message) as u64).unwrap();
            serialize::write_message(w, message)
        }
    }))
}

impl Capitanable for Ranking {
    fn write_to_cap<W: Write>(&self, w: &mut W, mode:Mode) -> Dx16Result<()> {
        let mut message = Builder::new_default();
        {
            let mut ranking = message.init_root::<ranking::Builder>();
            ranking.set_url(&*self.url);
            ranking.set_pagerank(self.pagerank);
            ranking.set_duration(self.duration);
        }
        write_message(&message, w, mode)
    }
}

impl Capitanable for UserVisits {
    fn write_to_cap<W: Write>(&self, w: &mut W, mode:Mode) -> Dx16Result<()> {
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
        write_message(&message, w, mode)
    }
}
