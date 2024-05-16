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
