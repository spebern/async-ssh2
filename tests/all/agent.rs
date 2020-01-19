use async_ssh2::Session;
use tokio;

#[tokio::test]
async fn smoke() {
    let sess = Session::new().unwrap();
    let mut agent = sess.agent().unwrap();
    agent.connect().await.unwrap();
    agent.list_identities().unwrap();
    {
        let a = agent.identities().unwrap();
        let i1 = &a[0];
        assert!(agent.userauth("foo", &i1).await.is_err());
    }
    agent.disconnect().await.unwrap();
}
