CREATE TABLE IF NOT EXISTS `users` (
    `trap_account_name` VARCHAR(100) NOT NULL PRIMARY KEY,
    `atcoder_account_name` VARCHAR(100),
    `atcoder_rating` INT,
    `heuristic_rating` INT,
    `is_algo_team` BOOLEAN,
    `is_active` BOOLEAN,
    `grade` VARCHAR(10)
);
