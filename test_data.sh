#!/bin/bash
API="http://localhost:8080"

# Register user 2
RES2=$(curl -s -X POST $API/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"alice123456","display_name":"Alice Dev"}')
echo "User 2: $RES2"
TOKEN2=$(echo $RES2 | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])" 2>/dev/null)

# Login as testuser
RES1=$(curl -s -X POST $API/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123456"}')
TOKEN1=$(echo $RES1 | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])" 2>/dev/null)
echo "Token1: $TOKEN1"

# Create snippet 1
curl -s -X POST $API/api/snippets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN1" \
  -d '{"title":"Python Hello World","description":"A simple Python example","language":"python","is_public":true,"tags":["python","beginner"],"files":[{"filename":"main.py","content":"def greet(name):\n    return f\"Hello, {name}!\"\n\nprint(greet(\"World\"))","language":"python"},{"filename":"utils.py","content":"import os\n\ndef get_env(key):\n    return os.environ.get(key)","language":"python"}]}'
echo ""

# Create snippet 2
curl -s -X POST $API/api/snippets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN1" \
  -d '{"title":"Rust Fibonacci","description":"Fibonacci sequence in Rust","language":"rust","is_public":true,"tags":["rust","algorithm"],"files":[{"filename":"main.rs","content":"fn fibonacci(n: u64) -> u64 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n-1) + fibonacci(n-2),\n    }\n}\n\nfn main() {\n    for i in 0..10 {\n        println!(\"fib({}) = {}\", i, fibonacci(i));\n    }\n}","language":"rust"}]}'
echo ""

# Create snippet 3 by alice
curl -s -X POST $API/api/snippets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN2" \
  -d '{"title":"JavaScript Array Utils","description":"Useful array utility functions","language":"javascript","is_public":true,"tags":["javascript","utils"],"files":[{"filename":"arrayUtils.js","content":"const unique = arr => [...new Set(arr)];\nconst flatten = arr => arr.flat(Infinity);\nconst chunk = (arr, size) => {\n  const result = [];\n  for (let i = 0; i < arr.length; i += size) {\n    result.push(arr.slice(i, i + size));\n  }\n  return result;\n};\n\nmodule.exports = { unique, flatten, chunk };","language":"javascript"}]}'
echo ""

# Create snippet 4
curl -s -X POST $API/api/snippets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN2" \
  -d '{"title":"SQL Cheat Sheet","description":"Common SQL queries for reference","language":"sql","is_public":true,"tags":["sql","database"],"files":[{"filename":"queries.sql","content":"-- Select with join\nSELECT u.name, o.total\nFROM users u\nJOIN orders o ON u.id = o.user_id\nWHERE o.created_at > NOW() - INTERVAL 30 DAY;\n\n-- Aggregation\nSELECT category, COUNT(*), AVG(price)\nFROM products\nGROUP BY category\nHAVING COUNT(*) > 5;","language":"sql"}]}'
echo ""

# Create snippet 5
curl -s -X POST $API/api/snippets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN1" \
  -d '{"title":"CSS Flexbox Layout","description":"Responsive flexbox layout examples","language":"css","is_public":true,"tags":["css","layout","flexbox"],"files":[{"filename":"layout.css","content":".container {\n  display: flex;\n  flex-wrap: wrap;\n  gap: 16px;\n  padding: 20px;\n}\n\n.card {\n  flex: 1 1 300px;\n  border: 1px solid #ddd;\n  border-radius: 8px;\n  padding: 16px;\n  box-shadow: 0 2px 4px rgba(0,0,0,0.1);\n}\n\n@media (max-width: 768px) {\n  .card { flex: 1 1 100%; }\n}","language":"css"},{"filename":"index.html","content":"<!DOCTYPE html>\n<html>\n<head><link rel=\"stylesheet\" href=\"layout.css\"></head>\n<body>\n  <div class=\"container\">\n    <div class=\"card\">Card 1</div>\n    <div class=\"card\">Card 2</div>\n    <div class=\"card\">Card 3</div>\n  </div>\n</body>\n</html>","language":"html"}]}'
echo ""

echo "Done! Test data created."
