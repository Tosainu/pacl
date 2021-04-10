use url::Url;

use crate::error::Result;

pub fn normalize_repo_url(repo: &str) -> Result<Url> {
    if !repo.contains("://") {
        if let Some((user_and_host, path)) = repo.find('@').and_then(|at| {
            repo[at..]
                .find(':')
                .map(|colon| (&repo[..at + colon], &repo[at + colon + 1..]))
        }) {
            // user@host:... => ssh://user@host/...
            return Ok(Url::parse(&format!("ssh://{}/{}", user_and_host, path))?);
        } else {
            // hogehoge  => https://github.com/hogehoge
            // hoge/fuga => https://github.com/hoge/fuga
            let mut url = Url::parse("https://github.com")?;
            url.set_path(&repo);
            return Ok(url);
        }
    };

    Ok(Url::parse(&repo)?)
}

#[test]
fn test_parse_repo_url() {
    assert_eq!(
        normalize_repo_url("user@host:hoge/fuga".to_owned()).unwrap(),
        Url::parse("ssh://user@host/hoge/fuga").unwrap()
    );
    assert_eq!(
        normalize_repo_url("hoge/fuga".to_owned()).unwrap(),
        Url::parse("https://github.com/hoge/fuga").unwrap()
    );
    assert_eq!(
        normalize_repo_url("hogehoge".to_owned()).unwrap(),
        Url::parse("https://github.com/hogehoge").unwrap()
    );

    assert_eq!(
        normalize_repo_url("https://host/hoge/fuga".to_owned()).unwrap(),
        Url::parse("https://host/hoge/fuga").unwrap()
    );
    assert_eq!(
        normalize_repo_url("ssh://user@host/hoge/fuga".to_owned()).unwrap(),
        Url::parse("ssh://user@host/hoge/fuga").unwrap()
    );
}
