$env:DATABASE_URL = "postgres://rustpress:rustpress@localhost:5433/rustpress"
$env:REDIS_URL = "redis://localhost:6380"
$env:JWT_SECRET = "rustpress-jwt-secret-key-2024"
$env:HOST = "127.0.0.1"
$env:PORT = "9000"

Set-Location "C:\Users\Software Engineering\Desktop\rustpress"
& "C:\Temp\rustpress-build-6\release\rustpress.exe"
