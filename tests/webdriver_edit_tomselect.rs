use tokio::test as tokio_test;
mod test_setup;
use test_setup::prelude::*;
use tokio;

mod webdriver_tests {
    use std::time::Duration;

    use super::*;
    use fantoccini::{key::Key, Locator};

    #[tokio_test]
    async fn webdriver_edit() -> Result<(), fantoccini::error::CmdError> {    
        let (server_task, geckodriver, c) = setup(true).await.unwrap();

        // Open the comment list page
        c.goto("http://localhost:5555/admin/comment/list").await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), "http://localhost:5555/admin/comment/list");

        // Click on edit first element
        let css_selector = "a[href='/admin/comment/edit/1']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        assert!(url.as_ref().contains("http://localhost:5555/admin/comment/edit/1"));

        // Fill the form
        let css_selector = "input[name='comment']";
        let el = c.find(Locator::Css(css_selector.into())).await?;
        el.clear().await?;
        el.send_keys("test comment").await?;
        let css_selector = "input[name='user']";
        let el = c.find(Locator::Css(css_selector.into())).await?;
        el.clear().await?;
        el.send_keys("test@test.com").await?;

        let css_selector = "input[name='insert_date']";
        let el = c.find(Locator::Css(css_selector.into())).await?;
        c.execute(
            "arguments[0].value = '2023-04-02 10:36:00';",
            vec![
                serde_json::to_value(&el).unwrap()
            ]
        ).await?;

        let input_element = c.find(Locator::Css("select[name='post_id']")).await?;
        let parent_div = input_element.find(Locator::XPath("..")).await?;
        parent_div.click().await?;
        let first_text_input = parent_div.find(Locator::Css("input[type='text']")).await?;
        first_text_input.send_keys("Test 10").await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        first_text_input.send_keys(&Key::Enter).await?;

        // save
        let css_selector = "button[name='submitBtn']";
        c.find(Locator::Css(css_selector.into())).await?.click().await?;
        let url = c.current_url().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(url.as_ref().contains("http://localhost:5555/admin/comment/list"));

        // assert content of first row
        let table = c.find(Locator::Css("tbody")).await?;
        let table_rows = table.find_all(Locator::Css("tr")).await?;
        let row_text = table_rows[0].text().await?;
    
        let expected_texts = vec!["test comment", "test@test.com", "2023-04-02 10:36:00", "Test 10"];
        for text in expected_texts {
            assert!(row_text.contains(text));
        }

        teardown(server_task, geckodriver, c).await
    }
}
