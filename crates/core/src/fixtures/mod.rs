use std::path::Path;
use std::process::{Command, Stdio};

/// Pin repository config and environment for deterministic OIDs.
pub fn pin(repo: &Path) -> std::io::Result<()> {
    let mut runner = GitRunner::new(repo);
    runner.run(&["config", "core.autocrlf", "false"])?;
    runner.run(&["config", "core.eol", "lf"])?;
    runner.run(&["config", "core.ignorecase", "false"])?;
    runner.run(&["config", "commit.gpgsign", "false"])?;
    runner.run(&["config", "user.name", "Gitflowfy Test"])?;
    runner.run(&["config", "user.email", "test@gitflowfy.local"])?;
    Ok(())
}

/// Minimal git runner for fixtures.
struct GitRunner {
    repo: std::path::PathBuf,
    commit_counter: u64,
}

impl GitRunner {
    fn new(repo: &Path) -> Self {
        Self { repo: repo.to_path_buf(), commit_counter: 0 }
    }

    fn run(&mut self, args: &[&str]) -> std::io::Result<()> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("GIT_PAGER", "")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .env("GIT_AUTHOR_NAME", "Gitflowfy Test")
            .env("GIT_AUTHOR_EMAIL", "test@gitflowfy.local")
            .env("GIT_COMMITTER_NAME", "Gitflowfy Test")
            .env("GIT_COMMITTER_EMAIL", "test@gitflowfy.local")
            .env("GIT_AUTHOR_DATE", &format!("2024-01-01 00:00:{:02} +0000", self.commit_counter))
            .env("GIT_COMMITTER_DATE", &format!("2024-01-01 00:00:{:02} +0000", self.commit_counter));
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(std::io::Error::other(String::from_utf8_lossy(&output.stderr)));
        }
        self.commit_counter += 1;
        Ok(())
    }
}

/// Build a linear fixture: one branch, no merges, every commit exactly one parent.
pub fn build_linear(path: &Path, commits: usize) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    for i in 0..commits {
        let file = path.join(format!("file_{}.txt", i));
        std::fs::write(&file, format!("content {}", i))?;
        runner.run(&["add", "."])?;
        runner.run(&["commit", "-m", &format!("commit {}", i)])?;
    }
    Ok(())
}

/// Build a merged fixture: one merge commit with exactly two parents.
pub fn build_merged(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    // Create base commit
    std::fs::write(path.join("base.txt"), "base")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "base"])?;

    // Create branch A
    runner.run(&["checkout", "-b", "branch-a"])?;
    std::fs::write(path.join("a.txt"), "a")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "commit A"])?;

    // Create branch B from base
    runner.run(&["checkout", "master"])?;
    runner.run(&["checkout", "-b", "branch-b"])?;
    std::fs::write(path.join("b.txt"), "b")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "commit B"])?;

    // Merge B into A with --no-ff
    runner.run(&["checkout", "branch-a"])?;
    runner.run(&["merge", "--no-ff", "branch-b", "-m", "merge A+B"])?;
    Ok(())
}

/// Build an octopus fixture: merge commit with three parents.
pub fn build_octopus(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    std::fs::write(path.join("base.txt"), "base")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "base"])?;

    // Create three branches
    for (name, content) in [("branch-1", "1"), ("branch-2", "2"), ("branch-3", "3")] {
        runner.run(&["checkout", "-b", name])?;
        std::fs::write(path.join(format!("{}.txt", name)), content)?;
        runner.run(&["add", "."])?;
        runner.run(&["commit", "-m", &format!("commit {}", name)])?;
        runner.run(&["checkout", "master"])?;
    }

    // Octopus merge all three into master
    runner.run(&["merge", "--no-ff", "branch-1", "branch-2", "branch-3", "-m", "octopus merge"])?;
    Ok(())
}

/// Build an orphan fixture: two branches with no merge base.
pub fn build_orphan(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    // First root commit on master
    std::fs::write(path.join("root1.txt"), "root1")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "root 1"])?;
    runner.run(&["branch", "orphan-1"])?;

    // Create orphan branch with no shared history
    runner.run(&["checkout", "--orphan", "orphan-2"])?;
    // Clear index
    runner.run(&["rm", "-rf", "."])?;
    std::fs::write(path.join("root2.txt"), "root2")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "root 2"])?;

    // Back to master
    runner.run(&["checkout", "master"])?;
    Ok(())
}

/// Build a detached HEAD fixture.
pub fn build_detached(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    std::fs::write(path.join("commit.txt"), "commit")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "commit"])?;

    // Detach HEAD at the commit
    runner.run(&["checkout", "--detach", "HEAD"])?;
    Ok(())
}

/// Build an empty fixture: no commits, unborn HEAD.
pub fn build_empty(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;
    // No commits - HEAD is unborn
    Ok(())
}

/// Build a fixture whose path contains spaces and non-ASCII characters.
pub fn build_path_edge(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let mut runner = GitRunner::new(path);
    runner.run(&["init"])?;
    pin(path)?;

    // File with spaces and non-ASCII in name
    let file = path.join("archivo con espacios y ñoño.txt");
    std::fs::write(&file, "contenido")?;
    runner.run(&["add", "."])?;
    runner.run(&["commit", "-m", "commit con espacios y ñoño"])?;

    Ok(())
}