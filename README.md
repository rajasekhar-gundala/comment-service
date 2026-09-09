# MarkReply 💬

MarkReply is a blazing-fast, self-hosted comment engine built in Rust. Designed for modern web frameworks, it provides a universal JavaScript widget and a fully automated SQLite backend, completely eliminating the need for heavy external dependencies.

## ✨ Features

* **Universal Widget:** Drop a single `<script>` tag into Astro, Next.js, WordPress, or plain HTML.
* **Threaded Conversations:** Built-in support for nested replies to keep discussions organized.
* **Zero-Touch Database:** Fully automated SQLite setup with build-time and run-time migrations.
* **Admin Auto-Approval:** The site owner's comments and replies bypass moderation and are instantly published.
* **Spam Protection:** Invisible honeypot fields to silently trap basic bot submissions.

## 🚀 Quick Start Deployment

MarkReply is designed to be entirely self-contained. The SQLite schema is automatically generated and migrated on startup, meaning zero database configuration is required.

**Docker Compose (Recommended)**
Create a `docker-compose.yml` file to manage your environment:

```yaml
services:
  markreply:
    image: ghcr.io/rajasekhar-gundala/markreply:latest
    build: .
    container_name: markreply
	restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=sqlite://data/comments.db?mode=rwc
      - ADMIN_TOKEN=your-super-secret-admin-key
    volumes:
      - ./data:/app/data

```

Start the engine by running `docker compose up -d`. The `./data` volume ensures your SQLite database persists safely across container restarts.

**Standard Docker Run**
If you prefer running the container directly without Compose, use the following command:

```bash
docker build -t markreply-core .

docker run -d \
  -p 3000:3000 \
  -e DATABASE_URL="sqlite://data/comments.db?mode=rwc" \
  -e ADMIN_TOKEN="your-super-secret-admin-key" \
  -v $(pwd)/data:/app/data \
  --name markreply-engine \
  markreply-core

```

## 💻 Frontend Integration

Once your backend is up and running, adding comments to your site is as simple as injecting the widget. Place this snippet exactly where you want the discussion thread to appear on your page:

<div class="callout callout-info">
  <p>Replace comments.example.com with your real domain name.</p>
</div>

```html
<!-- The data-slug attribute groups comments for specific pages or blog posts -->
<div id="markreply-comments"></div>
<script src="http://comments.example.com" async></script>

```
