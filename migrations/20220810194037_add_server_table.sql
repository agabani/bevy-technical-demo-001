create table server
(
    id         bigserial,
    ip_address text
);

create unique index index_name
    on server (ip_address);
