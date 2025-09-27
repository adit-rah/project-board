use git2::{Repository, Branch, BranchType, ObjectType, Signature, StatusOptions};
use anyhow::{Result, Context, bail};
use std::path::Path;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path)
            .context("Not in a git repository. Please run 'pb init' in a git repository.")?;
        
        Ok(GitRepo { repo })
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        // Get the current HEAD commit
        let head = self.repo.head()?;
        let target_commit = head.peel_to_commit()?;
        
        // Create the new branch
        self.repo.branch(branch_name, &target_commit, false)
            .context(format!("Failed to create branch '{}'", branch_name))?;
        
        Ok(())
    }

    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        // Find the branch
        let branch = self.repo.find_branch(branch_name, BranchType::Local)
            .context(format!("Branch '{}' not found", branch_name))?;
        
        // Get the tree for the branch
        let commit = branch.get().peel_to_commit()?;
        let tree = commit.tree()?;
        
        // Checkout the branch
        self.repo.checkout_tree(tree.as_object(), None)?;
        
        // Set HEAD to point to the branch
        self.repo.set_head(&format!("refs/heads/{}", branch_name))?;
        
        Ok(())
    }

    pub fn has_staged_changes(&self) -> Result<bool> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_ignored(false);
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        
        for status in statuses.iter() {
            let flags = status.status();
            if flags.is_index_new() || flags.is_index_modified() || flags.is_index_deleted() || flags.is_index_renamed() || flags.is_index_typechange() {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        // Get the current index
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        // Get the current HEAD
        let head = self.repo.head()?;
        let parent_commit = head.peel_to_commit()?;
        
        // Get signature
        let signature = self.get_signature()?;
        
        // Create the commit
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        
        Ok(())
    }

    pub fn push_branch(&self, branch_name: &str) -> Result<()> {
        // For now, just print that we would push
        // In a real implementation, we'd need to handle authentication
        println!("🔄 Pushing branch '{}' (git push simulation)", branch_name);
        
        // In a real implementation:
        // let mut remote = self.repo.find_remote("origin")?;
        // let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        // remote.push(&[&refspec], None)?;
        
        Ok(())
    }

    pub fn get_current_branch(&self) -> Result<Option<String>> {
        let head = self.repo.head()?;
        
        if head.is_branch() {
            let branch_name = head.shorthand()
                .ok_or_else(|| anyhow::anyhow!("Failed to get branch name"))?;
            Ok(Some(branch_name.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn get_remote_url(&self) -> Result<Option<String>> {
        let remote = self.repo.find_remote("origin");
        match remote {
            Ok(remote) => {
                let url = remote.url()
                    .ok_or_else(|| anyhow::anyhow!("Remote URL is not valid UTF-8"))?;
                Ok(Some(url.to_string()))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn is_clean_working_directory(&self) -> Result<bool> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_ignored(false);
        
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        Ok(statuses.is_empty())
    }

    fn get_signature(&self) -> Result<Signature> {
        // Try to get signature from git config
        let config = self.repo.config()?;
        
        let name = config.get_string("user.name")
            .unwrap_or_else(|_| "ProjectBoard User".to_string());
        let email = config.get_string("user.email")
            .unwrap_or_else(|_| "user@projectboard.dev".to_string());
        
        Ok(Signature::now(&name, &email)?)
    }
}
