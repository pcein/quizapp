
sqlite3 $1 <<EOS
	CREATE TABLE IF NOT EXISTS quiz_entry (
		team_name TEXT NOT NULL,
		qn_num INTEGER,
		SCORE INTEGER,
		UNIQUE (team_name, qn_num)
	);
EOS

