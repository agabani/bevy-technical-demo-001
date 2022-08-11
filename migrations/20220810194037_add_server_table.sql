create table server
(
    id         bigserial,
    public_id  uuid                     not null,
    last_seen  timestamp with time zone not null,
    ip_address text                     not null,
    port       integer                  not null
);

create unique index server_public_id_uindex
    on server (public_id);

create unique index server_ip_address_port_uindex
    on server (ip_address, port);

