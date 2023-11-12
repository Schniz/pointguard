CREATE OR REPLACE VIEW running_workers AS
SELECT DISTINCT application_name
FROM pg_stat_activity
WHERE application_name LIKE 'pointguard:%';
