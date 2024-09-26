--- 2024.09.26 22:12
--- Remove the autoincrement primary key from the id column
--- Make full_path the primary key

-- Create a temporary table to store the data
create table repos_temp
(
    created_at timestamp default CURRENT_TIMESTAMP not null,
    updated_at timestamp default CURRENT_TIMESTAMP not null,
    id         integer primary key,
    host       text                                not null,
    repo       text                                not null,
    owner      text                                not null,
    remote_url text                                not null,
    base_dir   text                                not null,
    full_path  text                                not null
);

-- Copy the data from the original table to the temporary table
insert into repos_temp
select created_at, updated_at, id, host, repo, owner, remote_url, base_dir, full_path
from repos;

-- Drop the original table
drop table repos;

-- Create the new table with the primary key on full_path
create table repos
(
    created_at timestamp default CURRENT_TIMESTAMP not null,
    updated_at timestamp default CURRENT_TIMESTAMP not null,
    host       text                                not null,
    repo       text                                not null,
    owner      text                                not null,
    remote_url text                                not null,
    base_dir   text                                not null,
    full_path  text                                not null,
    primary key (full_path)
);

-- Copy the data from the temporary table to the new table
--- distinct by full_path
insert into repos
select created_at, updated_at, host, repo, owner, remote_url, base_dir, full_path
from repos_temp
group by full_path;

-- Drop the temporary table
drop table repos_temp;
