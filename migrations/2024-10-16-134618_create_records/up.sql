-- Your SQL goes here
CREATE TABLE `repos`(
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`host` TEXT NOT NULL,
	`repo` TEXT NOT NULL,
	`owner` TEXT NOT NULL,
	`remote_url` TEXT NOT NULL,
	`base_dir` TEXT NOT NULL,
	`full_path` TEXT NOT NULL PRIMARY KEY
);

