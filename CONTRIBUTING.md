# Contributing to MarkReply

First off, thank you for considering contributing to MarkReply! 

To ensure the security, stability, and quality of the core engine, we maintain a strict review process. **Direct pushes to the `main` branch are disabled.** All changes must be submitted via a Pull Request (PR) and require explicit approval from the core maintainers before being merged.

## How to Contribute

### 1. Discuss Before You Code
Before starting work on a major feature, architectural change, or significant bug fix, **please open an Issue first**. This allows us to discuss the implementation details and ensures you don't waste time building something that might not align with the project's roadmap.

### 2. Fork and Branch
1. Fork the repository to your own GitHub account.
2. Clone your fork locally.
3. Create a new branch for your feature or bug fix:
   `git checkout -b feature/your-feature-name` or `git checkout -b fix/your-bug-fix`

### 3. Development Guidelines
* **Code Style:** Follow standard Rust formatting (`cargo fmt`) and ensure all linting passes (`cargo clippy`).
* **Database Migrations:** If your PR requires a database schema change, you must include the corresponding `.sql` migration file in the `/migrations` directory. 
* **Testing:** Ensure your changes do not break the Docker build process or the auto-migration steps.

### 4. Submit a Pull Request
* Push your branch to your forked repository.
* Open a Pull Request against the `main` branch of the upstream MarkReply repository.
* Fill out the PR template completely, explaining *why* the change is needed and *how* you implemented it.

### 5. The Review Process
Every Pull Request will be thoroughly reviewed. We evaluate PRs based on:
* Security implications (especially regarding the public API and database queries).
* Code cleanliness and adherence to the existing architecture.
* Minimal external dependencies.

**Note:** A PR is not guaranteed to be merged. The maintainers reserve the right to request changes, reject PRs that fall outside the scope of the project, or close stale PRs.

## Licensing 
MarkReply is licensed under the **AGPLv3**. By submitting a Pull Request, you agree to license your contribution under the same terms.