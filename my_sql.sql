SELECT 
	DISTINCT(category),
    description, 
    SUM(total_time_seconds) / 3600.0 as total_time_hours
FROM 
    timer
GROUP BY 
    description
ORDER BY 
    total_time_hours DESC;

select category, description, detail, start_time, end_time, total_time_seconds/3600.0 as hours from timer WHERE start_time >= DATE('now', '-1 day');

SELECT 
	DISTINCT(category), 
    description, 
    SUM(total_time_seconds) / 3600.0 as total_time_hours 
FROM 
    timer
WHERE start_time >= DATE('now', '-1 day')
GROUP BY 
    description
ORDER BY 
    total_time_hours DESC;