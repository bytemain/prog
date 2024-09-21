--- Create the table for the repos, use sqlite
--- This table will store the information about the repositories that are being managed by the console

create table repos
(
    created_at timestamp default CURRENT_TIMESTAMP not null,
    updated_at timestamp default CURRENT_TIMESTAMP not null,
    id         integer primary key autoincrement,
    host       text                                not null,
    repo       text                                not null,
    owner      text                                not null,
    remote_url text                                not null,
    base_dir   text                                not null
)