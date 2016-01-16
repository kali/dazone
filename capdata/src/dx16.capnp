@0xc55ce7f9ad3a3101;

struct Ranking {
    url @0: Text;
    pagerank @1: UInt64;
    duration @2: UInt64;
}

struct UserVisits {
    sourceIP @0: Text;
    destURL @1: Text;
    visitDate @2: Text;
    adRevenue @3: Float32;
    userAgent @4: Text;
    countryCode @5: Text;
    languageCode @6: Text;
    searchWord @7: Text;
    duration @8: UInt64;
}
