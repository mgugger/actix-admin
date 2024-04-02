use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use std::time::Duration;

    use super::*;
    use fantoccini::Locator;

    #[tokio_test]
    async fn webdriver_navigation() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(true).await.unwrap();

        // Open the index page
        c.goto("http://localhost:5555/admin/").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/");

        // Click on Post / the first entity
        c.find(Locator::LinkText("Post")).await?.click().await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/post/list");

        // assert 10 elements in the table
        let table = c.find(Locator::Css("tbody")).await?;
        let row_count = table.find_all(Locator::Css("tr")).await?.len();
        assert_eq!(row_count, 10, "Expected default 10 rows in the table");

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
        let first_tr = c.find(Locator::Css("tbody tr:first-child")).await?;
        let fourth_td = first_tr.find(Locator::Css("td:nth-child(3)")).await?;
        let td_text = fourth_td.text().await?;
        assert_eq!(td_text.trim(), "Test 41", "Text in the 4th td is not as expected");

        // change entities per page
        let dropdown = c.find(Locator::Css("select#entities_per_page")).await?;
        dropdown.select_by_value("100").await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("entities_per_page=100"));
        tokio::time::sleep(Duration::from_secs(1)).await;
        let table = c.find(Locator::Css("tbody")).await?;
        let row_count = table.find_all(Locator::Css("tr")).await?.len();
        assert_eq!(row_count, 100, "Expected default 100 rows in the table");

        // search for a specific row
        let search_input = c.find(Locator::Css("input#search")).await?;
        search_input.send_keys("Test 188").await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let table = c.find(Locator::Css("tbody")).await?;
        let row_count = table.find_all(Locator::Css("tr")).await?.len();
        assert_eq!(row_count, 1, "Expected a single row in the table");

        teardown(server_task, geckodriver, c).await
    }
}
