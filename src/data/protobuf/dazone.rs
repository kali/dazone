// This file is generated. Do not edit
// @generated

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,Default)]
pub struct Ranking {
    // message fields
    url: ::protobuf::SingularField<::std::string::String>,
    pagerank: ::std::option::Option<u64>,
    duration: ::std::option::Option<u64>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Ranking {
    pub fn new() -> Ranking {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Ranking {
        static mut instance: ::protobuf::lazy::Lazy<Ranking> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Ranking,
        };
        unsafe {
            instance.get(|| {
                Ranking {
                    url: ::protobuf::SingularField::none(),
                    pagerank: ::std::option::Option::None,
                    duration: ::std::option::Option::None,
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string url = 1;

    pub fn clear_url(&mut self) {
        self.url.clear();
    }

    pub fn has_url(&self) -> bool {
        self.url.is_some()
    }

    // Param is passed by value, moved
    pub fn set_url(&mut self, v: ::std::string::String) {
        self.url = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_url<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.url.is_none() {
            self.url.set_default();
        };
        self.url.as_mut().unwrap()
    }

    // Take field
    pub fn take_url(&mut self) -> ::std::string::String {
        self.url.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_url<'a>(&'a self) -> &'a str {
        match self.url.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required uint64 pagerank = 2;

    pub fn clear_pagerank(&mut self) {
        self.pagerank = ::std::option::Option::None;
    }

    pub fn has_pagerank(&self) -> bool {
        self.pagerank.is_some()
    }

    // Param is passed by value, moved
    pub fn set_pagerank(&mut self, v: u64) {
        self.pagerank = ::std::option::Option::Some(v);
    }

    pub fn get_pagerank<'a>(&self) -> u64 {
        self.pagerank.unwrap_or(0)
    }

    // required uint64 duration = 3;

    pub fn clear_duration(&mut self) {
        self.duration = ::std::option::Option::None;
    }

    pub fn has_duration(&self) -> bool {
        self.duration.is_some()
    }

    // Param is passed by value, moved
    pub fn set_duration(&mut self, v: u64) {
        self.duration = ::std::option::Option::Some(v);
    }

    pub fn get_duration<'a>(&self) -> u64 {
        self.duration.unwrap_or(0)
    }
}

impl ::protobuf::Message for Ranking {
    fn is_initialized(&self) -> bool {
        if self.url.is_none() {
            return false;
        };
        if self.pagerank.is_none() {
            return false;
        };
        if self.duration.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.url));
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint64());
                    self.pagerank = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint64());
                    self.duration = ::std::option::Option::Some(tmp);
                },
                _ => {
                    try!(::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields()));
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.url.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in self.pagerank.iter() {
            my_size += ::protobuf::rt::value_size(2, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        for value in self.duration.iter() {
            my_size += ::protobuf::rt::value_size(3, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.url.as_ref() {
            try!(os.write_string(1, &v));
        };
        if let Some(v) = self.pagerank {
            try!(os.write_uint64(2, v));
        };
        if let Some(v) = self.duration {
            try!(os.write_uint64(3, v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<Ranking>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Ranking {
    fn new() -> Ranking {
        Ranking::new()
    }

    fn descriptor_static(_: ::std::option::Option<Ranking>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "url",
                    Ranking::has_url,
                    Ranking::get_url,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_u64_accessor(
                    "pagerank",
                    Ranking::has_pagerank,
                    Ranking::get_pagerank,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_u64_accessor(
                    "duration",
                    Ranking::has_duration,
                    Ranking::get_duration,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Ranking>(
                    "Ranking",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Ranking {
    fn clear(&mut self) {
        self.clear_url();
        self.clear_pagerank();
        self.clear_duration();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Ranking {
    fn eq(&self, other: &Ranking) -> bool {
        self.url == other.url &&
        self.pagerank == other.pagerank &&
        self.duration == other.duration &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Ranking {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,Default)]
pub struct UserVisits {
    // message fields
    sourceIP: ::protobuf::SingularField<::std::string::String>,
    destURL: ::protobuf::SingularField<::std::string::String>,
    visitDate: ::protobuf::SingularField<::std::string::String>,
    adRevenue: ::std::option::Option<f32>,
    userAgent: ::protobuf::SingularField<::std::string::String>,
    countryCode: ::protobuf::SingularField<::std::string::String>,
    languageCode: ::protobuf::SingularField<::std::string::String>,
    searchWord: ::protobuf::SingularField<::std::string::String>,
    duration: ::std::option::Option<u64>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl UserVisits {
    pub fn new() -> UserVisits {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static UserVisits {
        static mut instance: ::protobuf::lazy::Lazy<UserVisits> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const UserVisits,
        };
        unsafe {
            instance.get(|| {
                UserVisits {
                    sourceIP: ::protobuf::SingularField::none(),
                    destURL: ::protobuf::SingularField::none(),
                    visitDate: ::protobuf::SingularField::none(),
                    adRevenue: ::std::option::Option::None,
                    userAgent: ::protobuf::SingularField::none(),
                    countryCode: ::protobuf::SingularField::none(),
                    languageCode: ::protobuf::SingularField::none(),
                    searchWord: ::protobuf::SingularField::none(),
                    duration: ::std::option::Option::None,
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string sourceIP = 1;

    pub fn clear_sourceIP(&mut self) {
        self.sourceIP.clear();
    }

    pub fn has_sourceIP(&self) -> bool {
        self.sourceIP.is_some()
    }

    // Param is passed by value, moved
    pub fn set_sourceIP(&mut self, v: ::std::string::String) {
        self.sourceIP = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_sourceIP<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.sourceIP.is_none() {
            self.sourceIP.set_default();
        };
        self.sourceIP.as_mut().unwrap()
    }

    // Take field
    pub fn take_sourceIP(&mut self) -> ::std::string::String {
        self.sourceIP.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_sourceIP<'a>(&'a self) -> &'a str {
        match self.sourceIP.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string destURL = 2;

    pub fn clear_destURL(&mut self) {
        self.destURL.clear();
    }

    pub fn has_destURL(&self) -> bool {
        self.destURL.is_some()
    }

    // Param is passed by value, moved
    pub fn set_destURL(&mut self, v: ::std::string::String) {
        self.destURL = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_destURL<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.destURL.is_none() {
            self.destURL.set_default();
        };
        self.destURL.as_mut().unwrap()
    }

    // Take field
    pub fn take_destURL(&mut self) -> ::std::string::String {
        self.destURL.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_destURL<'a>(&'a self) -> &'a str {
        match self.destURL.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string visitDate = 3;

    pub fn clear_visitDate(&mut self) {
        self.visitDate.clear();
    }

    pub fn has_visitDate(&self) -> bool {
        self.visitDate.is_some()
    }

    // Param is passed by value, moved
    pub fn set_visitDate(&mut self, v: ::std::string::String) {
        self.visitDate = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_visitDate<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.visitDate.is_none() {
            self.visitDate.set_default();
        };
        self.visitDate.as_mut().unwrap()
    }

    // Take field
    pub fn take_visitDate(&mut self) -> ::std::string::String {
        self.visitDate.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_visitDate<'a>(&'a self) -> &'a str {
        match self.visitDate.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required float adRevenue = 4;

    pub fn clear_adRevenue(&mut self) {
        self.adRevenue = ::std::option::Option::None;
    }

    pub fn has_adRevenue(&self) -> bool {
        self.adRevenue.is_some()
    }

    // Param is passed by value, moved
    pub fn set_adRevenue(&mut self, v: f32) {
        self.adRevenue = ::std::option::Option::Some(v);
    }

    pub fn get_adRevenue<'a>(&self) -> f32 {
        self.adRevenue.unwrap_or(0.)
    }

    // required string userAgent = 5;

    pub fn clear_userAgent(&mut self) {
        self.userAgent.clear();
    }

    pub fn has_userAgent(&self) -> bool {
        self.userAgent.is_some()
    }

    // Param is passed by value, moved
    pub fn set_userAgent(&mut self, v: ::std::string::String) {
        self.userAgent = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_userAgent<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.userAgent.is_none() {
            self.userAgent.set_default();
        };
        self.userAgent.as_mut().unwrap()
    }

    // Take field
    pub fn take_userAgent(&mut self) -> ::std::string::String {
        self.userAgent.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_userAgent<'a>(&'a self) -> &'a str {
        match self.userAgent.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string countryCode = 6;

    pub fn clear_countryCode(&mut self) {
        self.countryCode.clear();
    }

    pub fn has_countryCode(&self) -> bool {
        self.countryCode.is_some()
    }

    // Param is passed by value, moved
    pub fn set_countryCode(&mut self, v: ::std::string::String) {
        self.countryCode = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_countryCode<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.countryCode.is_none() {
            self.countryCode.set_default();
        };
        self.countryCode.as_mut().unwrap()
    }

    // Take field
    pub fn take_countryCode(&mut self) -> ::std::string::String {
        self.countryCode.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_countryCode<'a>(&'a self) -> &'a str {
        match self.countryCode.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string languageCode = 7;

    pub fn clear_languageCode(&mut self) {
        self.languageCode.clear();
    }

    pub fn has_languageCode(&self) -> bool {
        self.languageCode.is_some()
    }

    // Param is passed by value, moved
    pub fn set_languageCode(&mut self, v: ::std::string::String) {
        self.languageCode = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_languageCode<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.languageCode.is_none() {
            self.languageCode.set_default();
        };
        self.languageCode.as_mut().unwrap()
    }

    // Take field
    pub fn take_languageCode(&mut self) -> ::std::string::String {
        self.languageCode.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_languageCode<'a>(&'a self) -> &'a str {
        match self.languageCode.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string searchWord = 8;

    pub fn clear_searchWord(&mut self) {
        self.searchWord.clear();
    }

    pub fn has_searchWord(&self) -> bool {
        self.searchWord.is_some()
    }

    // Param is passed by value, moved
    pub fn set_searchWord(&mut self, v: ::std::string::String) {
        self.searchWord = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_searchWord<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.searchWord.is_none() {
            self.searchWord.set_default();
        };
        self.searchWord.as_mut().unwrap()
    }

    // Take field
    pub fn take_searchWord(&mut self) -> ::std::string::String {
        self.searchWord.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_searchWord<'a>(&'a self) -> &'a str {
        match self.searchWord.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required uint64 duration = 9;

    pub fn clear_duration(&mut self) {
        self.duration = ::std::option::Option::None;
    }

    pub fn has_duration(&self) -> bool {
        self.duration.is_some()
    }

    // Param is passed by value, moved
    pub fn set_duration(&mut self, v: u64) {
        self.duration = ::std::option::Option::Some(v);
    }

    pub fn get_duration<'a>(&self) -> u64 {
        self.duration.unwrap_or(0)
    }
}

impl ::protobuf::Message for UserVisits {
    fn is_initialized(&self) -> bool {
        if self.sourceIP.is_none() {
            return false;
        };
        if self.destURL.is_none() {
            return false;
        };
        if self.visitDate.is_none() {
            return false;
        };
        if self.adRevenue.is_none() {
            return false;
        };
        if self.userAgent.is_none() {
            return false;
        };
        if self.countryCode.is_none() {
            return false;
        };
        if self.languageCode.is_none() {
            return false;
        };
        if self.searchWord.is_none() {
            return false;
        };
        if self.duration.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.sourceIP));
                },
                2 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.destURL));
                },
                3 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.visitDate));
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed32 {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_float());
                    self.adRevenue = ::std::option::Option::Some(tmp);
                },
                5 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.userAgent));
                },
                6 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.countryCode));
                },
                7 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.languageCode));
                },
                8 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.searchWord));
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint64());
                    self.duration = ::std::option::Option::Some(tmp);
                },
                _ => {
                    try!(::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields()));
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.sourceIP.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in self.destURL.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        for value in self.visitDate.iter() {
            my_size += ::protobuf::rt::string_size(3, &value);
        };
        if self.adRevenue.is_some() {
            my_size += 5;
        };
        for value in self.userAgent.iter() {
            my_size += ::protobuf::rt::string_size(5, &value);
        };
        for value in self.countryCode.iter() {
            my_size += ::protobuf::rt::string_size(6, &value);
        };
        for value in self.languageCode.iter() {
            my_size += ::protobuf::rt::string_size(7, &value);
        };
        for value in self.searchWord.iter() {
            my_size += ::protobuf::rt::string_size(8, &value);
        };
        for value in self.duration.iter() {
            my_size += ::protobuf::rt::value_size(9, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.sourceIP.as_ref() {
            try!(os.write_string(1, &v));
        };
        if let Some(v) = self.destURL.as_ref() {
            try!(os.write_string(2, &v));
        };
        if let Some(v) = self.visitDate.as_ref() {
            try!(os.write_string(3, &v));
        };
        if let Some(v) = self.adRevenue {
            try!(os.write_float(4, v));
        };
        if let Some(v) = self.userAgent.as_ref() {
            try!(os.write_string(5, &v));
        };
        if let Some(v) = self.countryCode.as_ref() {
            try!(os.write_string(6, &v));
        };
        if let Some(v) = self.languageCode.as_ref() {
            try!(os.write_string(7, &v));
        };
        if let Some(v) = self.searchWord.as_ref() {
            try!(os.write_string(8, &v));
        };
        if let Some(v) = self.duration {
            try!(os.write_uint64(9, v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<UserVisits>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for UserVisits {
    fn new() -> UserVisits {
        UserVisits::new()
    }

    fn descriptor_static(_: ::std::option::Option<UserVisits>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "sourceIP",
                    UserVisits::has_sourceIP,
                    UserVisits::get_sourceIP,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "destURL",
                    UserVisits::has_destURL,
                    UserVisits::get_destURL,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "visitDate",
                    UserVisits::has_visitDate,
                    UserVisits::get_visitDate,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_f32_accessor(
                    "adRevenue",
                    UserVisits::has_adRevenue,
                    UserVisits::get_adRevenue,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "userAgent",
                    UserVisits::has_userAgent,
                    UserVisits::get_userAgent,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "countryCode",
                    UserVisits::has_countryCode,
                    UserVisits::get_countryCode,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "languageCode",
                    UserVisits::has_languageCode,
                    UserVisits::get_languageCode,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "searchWord",
                    UserVisits::has_searchWord,
                    UserVisits::get_searchWord,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_u64_accessor(
                    "duration",
                    UserVisits::has_duration,
                    UserVisits::get_duration,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<UserVisits>(
                    "UserVisits",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for UserVisits {
    fn clear(&mut self) {
        self.clear_sourceIP();
        self.clear_destURL();
        self.clear_visitDate();
        self.clear_adRevenue();
        self.clear_userAgent();
        self.clear_countryCode();
        self.clear_languageCode();
        self.clear_searchWord();
        self.clear_duration();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for UserVisits {
    fn eq(&self, other: &UserVisits) -> bool {
        self.sourceIP == other.sourceIP &&
        self.destURL == other.destURL &&
        self.visitDate == other.visitDate &&
        self.adRevenue == other.adRevenue &&
        self.userAgent == other.userAgent &&
        self.countryCode == other.countryCode &&
        self.languageCode == other.languageCode &&
        self.searchWord == other.searchWord &&
        self.duration == other.duration &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for UserVisits {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x10, 0x73, 0x72, 0x63, 0x2f, 0x64, 0x61, 0x7a, 0x6f, 0x6e, 0x65, 0x2e, 0x70, 0x72, 0x6f,
    0x74, 0x6f, 0x22, 0x3a, 0x0a, 0x07, 0x52, 0x61, 0x6e, 0x6b, 0x69, 0x6e, 0x67, 0x12, 0x0b, 0x0a,
    0x03, 0x75, 0x72, 0x6c, 0x18, 0x01, 0x20, 0x02, 0x28, 0x09, 0x12, 0x10, 0x0a, 0x08, 0x70, 0x61,
    0x67, 0x65, 0x72, 0x61, 0x6e, 0x6b, 0x18, 0x02, 0x20, 0x02, 0x28, 0x04, 0x12, 0x10, 0x0a, 0x08,
    0x64, 0x75, 0x72, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x18, 0x03, 0x20, 0x02, 0x28, 0x04, 0x22, 0xb9,
    0x01, 0x0a, 0x0a, 0x55, 0x73, 0x65, 0x72, 0x56, 0x69, 0x73, 0x69, 0x74, 0x73, 0x12, 0x10, 0x0a,
    0x08, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x49, 0x50, 0x18, 0x01, 0x20, 0x02, 0x28, 0x09, 0x12,
    0x0f, 0x0a, 0x07, 0x64, 0x65, 0x73, 0x74, 0x55, 0x52, 0x4c, 0x18, 0x02, 0x20, 0x02, 0x28, 0x09,
    0x12, 0x11, 0x0a, 0x09, 0x76, 0x69, 0x73, 0x69, 0x74, 0x44, 0x61, 0x74, 0x65, 0x18, 0x03, 0x20,
    0x02, 0x28, 0x09, 0x12, 0x11, 0x0a, 0x09, 0x61, 0x64, 0x52, 0x65, 0x76, 0x65, 0x6e, 0x75, 0x65,
    0x18, 0x04, 0x20, 0x02, 0x28, 0x02, 0x12, 0x11, 0x0a, 0x09, 0x75, 0x73, 0x65, 0x72, 0x41, 0x67,
    0x65, 0x6e, 0x74, 0x18, 0x05, 0x20, 0x02, 0x28, 0x09, 0x12, 0x13, 0x0a, 0x0b, 0x63, 0x6f, 0x75,
    0x6e, 0x74, 0x72, 0x79, 0x43, 0x6f, 0x64, 0x65, 0x18, 0x06, 0x20, 0x02, 0x28, 0x09, 0x12, 0x14,
    0x0a, 0x0c, 0x6c, 0x61, 0x6e, 0x67, 0x75, 0x61, 0x67, 0x65, 0x43, 0x6f, 0x64, 0x65, 0x18, 0x07,
    0x20, 0x02, 0x28, 0x09, 0x12, 0x12, 0x0a, 0x0a, 0x73, 0x65, 0x61, 0x72, 0x63, 0x68, 0x57, 0x6f,
    0x72, 0x64, 0x18, 0x08, 0x20, 0x02, 0x28, 0x09, 0x12, 0x10, 0x0a, 0x08, 0x64, 0x75, 0x72, 0x61,
    0x74, 0x69, 0x6f, 0x6e, 0x18, 0x09, 0x20, 0x02, 0x28, 0x04, 0x4a, 0xf4, 0x06, 0x0a, 0x06, 0x12,
    0x04, 0x00, 0x00, 0x10, 0x01, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x00, 0x00, 0x04,
    0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x00, 0x08, 0x0f, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x01, 0x04, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x00, 0x04, 0x12, 0x03, 0x01, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00,
    0x05, 0x12, 0x03, 0x01, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12,
    0x03, 0x01, 0x14, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x01,
    0x1a, 0x1b, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x02, 0x04, 0x21, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x04, 0x12, 0x03, 0x02, 0x04, 0x0c, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12, 0x03, 0x02, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x02, 0x14, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x01, 0x03, 0x12, 0x03, 0x02, 0x1f, 0x20, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12,
    0x03, 0x03, 0x04, 0x21, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x03, 0x03,
    0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x05, 0x12, 0x03, 0x03, 0x0d, 0x13,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x01, 0x12, 0x03, 0x03, 0x14, 0x1c, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x03, 0x12, 0x03, 0x03, 0x1f, 0x20, 0x0a, 0x0a, 0x0a, 0x02,
    0x04, 0x01, 0x12, 0x04, 0x06, 0x00, 0x10, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x01, 0x01, 0x12,
    0x03, 0x06, 0x08, 0x12, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x07, 0x04,
    0x21, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x04, 0x12, 0x03, 0x07, 0x04, 0x0c, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x05, 0x12, 0x03, 0x07, 0x0d, 0x13, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x07, 0x14, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x07, 0x1f, 0x20, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02,
    0x01, 0x12, 0x03, 0x08, 0x04, 0x20, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x04, 0x12,
    0x03, 0x08, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x05, 0x12, 0x03, 0x08,
    0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x01, 0x12, 0x03, 0x08, 0x14, 0x1b,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x03, 0x12, 0x03, 0x08, 0x1e, 0x1f, 0x0a, 0x0b,
    0x0a, 0x04, 0x04, 0x01, 0x02, 0x02, 0x12, 0x03, 0x09, 0x04, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x01, 0x02, 0x02, 0x04, 0x12, 0x03, 0x09, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02,
    0x02, 0x05, 0x12, 0x03, 0x09, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x01,
    0x12, 0x03, 0x09, 0x14, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x03, 0x12, 0x03,
    0x09, 0x20, 0x21, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x03, 0x12, 0x03, 0x0a, 0x04, 0x21,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x03, 0x04, 0x12, 0x03, 0x0a, 0x04, 0x0c, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x01, 0x02, 0x03, 0x05, 0x12, 0x03, 0x0a, 0x0d, 0x12, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x01, 0x02, 0x03, 0x01, 0x12, 0x03, 0x0a, 0x13, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x03, 0x03, 0x12, 0x03, 0x0a, 0x1f, 0x20, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x04,
    0x12, 0x03, 0x0b, 0x04, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x04, 0x04, 0x12, 0x03,
    0x0b, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x04, 0x05, 0x12, 0x03, 0x0b, 0x0d,
    0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x04, 0x01, 0x12, 0x03, 0x0b, 0x14, 0x1d, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x04, 0x03, 0x12, 0x03, 0x0b, 0x20, 0x21, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x01, 0x02, 0x05, 0x12, 0x03, 0x0c, 0x04, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x05, 0x04, 0x12, 0x03, 0x0c, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x05,
    0x05, 0x12, 0x03, 0x0c, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x05, 0x01, 0x12,
    0x03, 0x0c, 0x14, 0x1f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x05, 0x03, 0x12, 0x03, 0x0c,
    0x22, 0x23, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x06, 0x12, 0x03, 0x0d, 0x04, 0x25, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x06, 0x04, 0x12, 0x03, 0x0d, 0x04, 0x0c, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x06, 0x05, 0x12, 0x03, 0x0d, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x01, 0x02, 0x06, 0x01, 0x12, 0x03, 0x0d, 0x14, 0x20, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02,
    0x06, 0x03, 0x12, 0x03, 0x0d, 0x23, 0x24, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x07, 0x12,
    0x03, 0x0e, 0x04, 0x23, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x07, 0x04, 0x12, 0x03, 0x0e,
    0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x07, 0x05, 0x12, 0x03, 0x0e, 0x0d, 0x13,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x07, 0x01, 0x12, 0x03, 0x0e, 0x14, 0x1e, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x01, 0x02, 0x07, 0x03, 0x12, 0x03, 0x0e, 0x21, 0x22, 0x0a, 0x0b, 0x0a, 0x04,
    0x04, 0x01, 0x02, 0x08, 0x12, 0x03, 0x0f, 0x04, 0x21, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02,
    0x08, 0x04, 0x12, 0x03, 0x0f, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x08, 0x05,
    0x12, 0x03, 0x0f, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x08, 0x01, 0x12, 0x03,
    0x0f, 0x14, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x08, 0x03, 0x12, 0x03, 0x0f, 0x1f,
    0x20,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
