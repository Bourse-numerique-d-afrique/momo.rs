# Running MTN MoMo Callback Server on Port 443

Since MTN MoMo requires callbacks to be sent to port 443 (HTTPS), the server needs elevated privileges to bind to this privileged port. Here are the available options:

## Option 1: Run with sudo (Development)

```bash
sudo cargo run
```

**Pros:**
- Quick and simple for development
- No additional configuration needed

**Cons:**
- Runs the entire process as root
- Not recommended for production

## Option 2: Grant bind privileges to the binary

First, build the project:
```bash
cargo build
```

Grant the capability to bind to privileged ports:
```bash
sudo setcap 'cap_net_bind_service=+ep' target/debug/momo-callback-server
```

Then run normally:
```bash
cargo run
```

**Pros:**
- More secure than running as root
- Good for development and testing
- Process runs as regular user

**Cons:**
- Need to re-apply after each rebuild
- Binary-specific capability

## Option 3: Systemd Service (Production Recommended)

Create the service file at `/etc/systemd/system/momo-callback-server.service`:

```ini
[Unit]
Description=MTN MoMo Callback Server
After=network.target

[Service]
Type=simple
User=your-user
Group=your-group
ExecStart=/home/ondonda/rust/momo.rs/momo-callback-server/target/release/momo-callback-server
WorkingDirectory=/home/ondonda/rust/momo.rs/momo-callback-server
Restart=always
RestartSec=10

# Environment variables (if needed)
Environment=TLS_CERT_PATH=/home/ondonda/bna/certs/config/live/momotest.boursenumeriquedafrique.com/fullchain.pem
Environment=TLS_KEY_PATH=/home/ondonda/bna/certs/config/live/momotest.boursenumeriquedafrique.com/privkey.pem

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log

[Install]
WantedBy=multi-user.target
```

Enable and start the service:
```bash
# Build release version
cargo build --release

# Grant capabilities to the release binary
sudo setcap 'cap_net_bind_service=+ep' target/release/momo-callback-server

# Reload systemd configuration
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable momo-callback-server

# Start the service
sudo systemctl start momo-callback-server

# Check status
sudo systemctl status momo-callback-server

# View logs
sudo journalctl -u momo-callback-server -f
```

**Pros:**
- Automatic startup on boot
- Process management and restart on failure
- Proper logging integration
- Security hardening options
- Production-ready

**Cons:**
- More complex initial setup
- Requires systemd

## Option 4: Reverse Proxy (Alternative Approach)

If you prefer not to run your application with elevated privileges, you can:

1. Run your app on port 8443 (non-privileged)
2. Use nginx/traefik as a reverse proxy on port 443
3. Configure the proxy to forward requests to your app

Example nginx configuration:
```nginx
server {
    listen 443 ssl;
    server_name momotest.boursenumeriquedafrique.com;
    
    ssl_certificate /home/ondonda/bna/certs/config/live/momotest.boursenumeriquedafrique.com/fullchain.pem;
    ssl_certificate_key /home/ondonda/bna/certs/config/live/momotest.boursenumeriquedafrique.com/privkey.pem;
    
    location / {
        proxy_pass https://localhost:8443;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Recommendation

- **Development**: Use Option 1 (sudo cargo run)
- **Production**: Use Option 3 (systemd service)
- **High-security environments**: Consider Option 4 (reverse proxy)

## Troubleshooting

### Permission Denied Error
```
Server error: Permission denied (os error 13)
```
This means the process doesn't have permission to bind to port 443. Use one of the options above.

### Certificate Issues
Ensure certificate files exist and are readable:
```bash
ls -la /home/ondonda/bna/certs/config/live/momotest.boursenumeriquedafrique.com/
```

### Port Already in Use
Check what's using port 443:
```bash
sudo netstat -tlnp | grep :443
```