use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use super::*;
    use fantoccini::Locator;

    #[tokio_test]
    async fn test_with_webdriver() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(true).await.unwrap();

        // Open the index page
        c.goto("http://localhost:5555/admin/").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/");

        // Click on Post / the first entity
        c.find(Locator::LinkText("Post")).await?.click().await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/post/list");

        // click on a Show Element
        let link_locator = Locator::Css("table a:first-child".into());
        c.find(link_locator).await?.click().await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("http://localhost:5555/admin/post/show/1"));

        // click the back button
        c.find(Locator::LinkText("Back")).await?.click().await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("http://localhost:5555/admin/post/list"));

        // click on a pagination element
        let css_selector = "a.pagination-link[href='/admin/post/list?page=5']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("page=5"));

        teardown(server_task, geckodriver, c).await
    }
}
