CREATE TABLE titan_users
(
  id               INT AUTO_INCREMENT PRIMARY KEY,
  wcf_id           INT UNIQUE,
  legacy_player_id INT,
  rank_id          INT,
  username         VARCHAR(255) UNIQUE,
  password         VARCHAR(255),
  date_joined      DATETIME,
  orientation      INT,
  bct_e0           INT,
  bct_e1           INT,
  bct_e2           INT,
  bct_e3           INT,
  loa              INT,
  a15              INT,
  is_awol          BOOLEAN DEFAULT 0,
  date_created     DATETIME,
  date_modified    DATETIME,
  modified_by      INT,
  last_activity    DATETIME,
  is_active        BOOLEAN DEFAULT 1
);