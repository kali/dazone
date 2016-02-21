#!/usr/bin/perl -p

s/^2016-02-08T[^ ]+/type: c3.8xlarge nodes:  5/ ;
s/^2016-02-09T[^ ]+/type:  m3.xlarge nodes: 20/ ;
s/^2016-02-10T[^ ]+/type: m3.2xlarge nodes: 20/ ;
s/^2016-02-11T[^ ]+/type: c3.8xlarge nodes:  5/ ;
s/^2016-02-12T(08|09|10)[^ ]+/type:  m3.xlarge nodes: 20/ ;
s/^2016-02-12T(11|12:0)[^ ]+/type: c3.8xlarge nodes:  5/ ;
s/^2016-02-12T(12|13:0|13:1)[^ ]+/type: c3.2xlarge nodes: 10/ ;
s/^2016-02-12T(13|14)[^ ]+/type: m3.2xlarge nodes: 10/ ;
s/^2016-02-12T(15|16)[^ ]+/type: c3.8xlarge nodes:  5/ ;
s/^2016-02-13T(10|11|12)[^ ]+/type: c3.8xlarge nodes:  5/ ;
s/^2016-02-15T(08)[^ ]+/type: m3.2xlarge nodes: 10/ ;
s/^2016-02-15T(09|10|11)[^ ]+/type:  m3.xlarge nodes: 20/ ;

s/type:  m3.xlarge/type:  m3.xlarge ucost: 0.266/ ;
s/type: m3.2xlarge/type: m3.2xlarge ucost: 0.532/ ;
s/type:  c3.xlarge/type:  c3.xlarge ucost: 0.210/ ;
s/type: c3.2xlarge/type: c3.2xlarge ucost: 0.420/ ;
s/type: c3.8xlarge/type: c3.8xlarge ucost: 1.680/ ;
