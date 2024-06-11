use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use std::time::Duration;

    use super::*;
    use fantoccini::Locator;

    #[tokio_test]
    async fn webdriver_support() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(true).await.unwrap();

        // Open the index page
        c.goto("http://localhost:5555/admin/").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/");
        
        let html_source = c.source().await?;
        assert_eq!(html_source.contains("Card1"), false, "Expected no Card1 on the page");

        // Click on support question mark
        c.find(Locator::LinkText("Card Grid")).await?.click().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/my_card_grid");

        let html_source = c.source().await?;
        assert!(html_source.contains("Card1"), "Expected Card1 on the page");
        assert!(html_source.contains("Card2"), "Expected Card2 on the page");
        assert!(html_source.contains("Card3"), "Expected Card3 on the page");

        teardown(server_task, geckodriver, c).await
    }
}
