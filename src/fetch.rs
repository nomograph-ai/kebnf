use std::path::PathBuf;

const KERML_URL: &str = "https://raw.githubusercontent.com/Systems-Modeling/SysML-v2-Release/master/bnf/KerML-textual-bnf.kebnf";
const SYSML_URL: &str = "https://raw.githubusercontent.com/Systems-Modeling/SysML-v2-Release/master/bnf/SysML-textual-bnf.kebnf";

pub fn fetch_specs(verbose: bool) -> Result<Vec<PathBuf>, FetchError> {
    let cache_dir = get_cache_dir()?;
    std::fs::create_dir_all(&cache_dir)?;

    let mut paths = Vec::new();

    for (url, filename) in [
        (KERML_URL, "KerML-textual-bnf.kebnf"),
        (SYSML_URL, "SysML-textual-bnf.kebnf"),
    ] {
        let path = cache_dir.join(filename);

        if verbose {
            eprintln!("Fetching {} ...", url);
        }

        let response = reqwest::blocking::get(url)?;
        let content = response.text()?;
        std::fs::write(&path, &content)?;

        if verbose {
            eprintln!("Saved to {}", path.display());
        }

        paths.push(path);
    }

    Ok(paths)
}

pub fn get_cache_dir() -> Result<PathBuf, FetchError> {
    let proj_dirs = directories::ProjectDirs::from("ai", "nomograph", "kebnf")
        .ok_or_else(|| FetchError::NoCacheDir)?;
    Ok(proj_dirs.cache_dir().to_path_buf())
}

#[derive(Debug)]
pub enum FetchError {
    NoCacheDir,
    Io(std::io::Error),
    Http(reqwest::Error),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::NoCacheDir => write!(f, "Could not determine cache directory"),
            FetchError::Io(e) => write!(f, "IO error: {}", e),
            FetchError::Http(e) => write!(f, "HTTP error: {}", e),
        }
    }
}

impl std::error::Error for FetchError {}

impl From<std::io::Error> for FetchError {
    fn from(e: std::io::Error) -> Self {
        FetchError::Io(e)
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        FetchError::Http(e)
    }
}
