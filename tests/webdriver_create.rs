use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use std::time::Duration;

    use super::*;
    use fantoccini::Locator;

    #[tokio_test]
    async fn webdriver_create() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(false, false).await.unwrap();

        // Open the post list page
        c.goto("http://localhost:5555/admin/post/list").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/post/list");

        // Click on create
        let css_selector = "a[href='/admin/post/create']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/post/create");

        // Fill the form
        let css_selector = "input[name='title']";
        c.find(Locator::Css(css_selector.into())).await?.send_keys("testtitle").await?;
        let css_selector = "input[name='text']";
        c.find(Locator::Css(css_selector.into())).await?.send_keys("testtext").await?;
        let css_selector = c.find(Locator::Css("select[name='tea_mandatory']")).await?;
        css_selector.select_by_value("EverydayTea").await?;
        let css_selector = "input[name='insert_date']";
        c.find(Locator::Css(css_selector.into())).await?.send_keys("2024-04-02").await?;

        // save
        let css_selector = "button[name='submitBtn']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(url.as_ref().contains("http://localhost:5555/admin/post/list"));

        // assert 1 elements in the table
        let table = c.find(Locator::Css("tbody")).await?;
        let row_count = table.find_all(Locator::Css("tr")).await?.len();
        assert_eq!(row_count, 1, "Expected 1 row created in the table");
        let html_source = c.source().await?;
        
        assert!(html_source.contains("testtitle"), "Expected testtitle in the table");
        assert!(html_source.contains("testtext"), "Expected testtext in the table");
        assert!(html_source.contains("2024-04-02"), "Expected correct date in the table");
        assert!(html_source.contains("EverydayTea"), "Expected EverydayTea in the table");

        teardown(server_task, geckodriver, c).await
    }
}
