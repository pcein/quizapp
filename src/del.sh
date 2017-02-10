#!/bin/bash

if test $# -ne 3
then
   echo 'Usage: del.sh db_file team_name qn_num'
   exit 1
fi

db_file=$1
team_name=$2
qn_num=$3
sqlite3 $db_file <<EOS
	DELETE FROM quiz_entry 
    WHERE team_name="$team_name" AND qn_num=$qn_num;
EOS

