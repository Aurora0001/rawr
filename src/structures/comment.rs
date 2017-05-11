use serde_json::from_value;
use traits::{Votable, Created, Editable, Content, Commentable, Reportable, Stickable,
             Distinguishable, Approvable};
use structures::comment_list::CommentList;
use structures::subreddit::Subreddit;
use structures::user::User;
use client::RedditClient;
use responses::comment::{Comment as _Comment, CommentListing, NewComment};
use errors::APIError;

/// Structure representing a comment and its associated data (e.g. replies)
pub struct Comment<'a> {
    data: _Comment,
    client: &'a RedditClient,
    replies: CommentList<'a>,
}

impl<'a> Votable for Comment<'a> {
    fn score(&self) -> i64 {
        self.data.score
    }

    fn likes(&self) -> Option<bool> {
        self.data.likes
    }

    fn upvote(&self) -> Result<(), APIError> {
        self.vote(1)
    }

    fn downvote(&self) -> Result<(), APIError> {
        self.vote(-1)
    }

    fn cancel_vote(&self) -> Result<(), APIError> {
        self.vote(0)
    }
}

impl<'a> Created for Comment<'a> {
    fn created(&self) -> i64 {
        self.data.created
    }

    fn created_utc(&self) -> i64 {
        self.data.created_utc
    }
}

impl<'a> Editable for Comment<'a> {
    fn edited(&self) -> bool {
        match self.data.edited.as_bool() {
            Some(edited) => edited,
            None => true,
        }
    }

    fn edited_time(&self) -> Option<i64> {
        self.data.edited.as_i64()
    }

    fn edit(&mut self, text: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.data.name);
        let res = self.client.post_success("/api/editusertext", &body, false);
        if let Ok(()) = res {
            // TODO: should we update body_html?
            self.data.body = text.to_owned();
        }
        res
    }

    fn body(&self) -> Option<String> {
        Some(self.data.body.to_owned())
    }

    fn body_html(&self) -> Option<String> {
        Some(self.data.body_html.to_owned())
    }
}

impl<'a> Content for Comment<'a> {
    fn author(&self) -> User {
        User::new(self.client, &self.data.author)
    }

    fn author_flair_text(&self) -> Option<String> {
        self.data.author_flair_text.to_owned()
    }

    fn author_flair_css(&self) -> Option<String> {
        self.data.author_flair_css_class.to_owned()
    }

    fn subreddit(&self) -> Subreddit {
        Subreddit::create_new(self.client, &self.data.subreddit)
    }

    fn delete(self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/del", &body, false)
    }

    fn name(&self) -> &str {
        &self.data.name
    }
}

impl<'a> Approvable for Comment<'a> {
    fn approve(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/approve", &body, false)
    }

    fn remove(&self, spam: bool) -> Result<(), APIError> {
        let body = format!("id={}&spam={}", self.data.name, spam);
        self.client.post_success("/api/remove", &body, false)
    }

    fn ignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/ignore_reports", &body, false)
    }

    fn unignore_reports(&self) -> Result<(), APIError> {
        let body = format!("id={}", self.data.name);
        self.client.post_success("/api/unignore_reports", &body, false)
    }
}

impl<'a> Commentable<'a> for Comment<'a> {
    fn reply_count(&self) -> u64 {
        panic!("There is no effective way of getting the number of comment replies. You may have \
                to manually count with `replies().len()`, which may take some time.");
    }

    fn reply(&self, text: &str) -> Result<Comment, APIError> {
        let body = format!("api_type=json&text={}&thing_id={}",
                           self.client.url_escape(text.to_owned()),
                           self.name());
        self.client
            .post_json::<NewComment>("/api/comment", &body, false)
            .and_then(|res| {
                let data: Result<_, APIError> = res.json
                    .data
                    .things
                    .into_iter()
                    .next()
                    .ok_or_else(|| APIError::MissingField("things[0]"));
                Ok(Comment::new(self.client, data?.data))
            })
    }

    fn replies(self) -> Result<CommentList<'a>, APIError> {
        Ok(self.replies)
    }
}

impl<'a> Comment<'a> {
    /// Internal method. Use `Submission.replies()` or `Comment.replies()` to get a listing, then
    /// select the desired comment instead.
    pub fn new(client: &RedditClient, data: _Comment) -> Comment {
        let comments = if data.replies.is_object() {
            // TODO: avoid cloning here
            let listing = from_value::<CommentListing>(data.replies.clone()).unwrap();
            CommentList::new(client,
                             data.link_id.to_owned(),
                             data.name.to_owned(),
                             listing.data.children)
        } else {
            CommentList::empty(client)
        };

        Comment {
            client: client,
            data: data,
            replies: comments,
        }
    }

    /// Gets the full ID of the parent submission/comment (kind + id e.g. 't1_4te6jf')
    pub fn parent(&self) -> &str {
        &self.data.parent_id
    }

    /// Adds a reply to this comment's reply list. This is an internal method - to make the client
    /// reply to this post, use `Comment.reply(MESSAGE)`.
    pub fn add_reply(&mut self, item: Comment<'a>) {
        self.replies.add_reply(item);
    }

    fn vote(&self, dir: i8) -> Result<(), APIError> {
        let body = format!("dir={}&id={}", dir, self.data.name);
        self.client.post_success("/api/vote", &body, false)
    }
}

impl<'a> Reportable for Comment<'a> {
    fn report(&self, reason: &str) -> Result<(), APIError> {
        let body = format!("api_type=json&thing_id={}&reason={}",
                           self.data.name,
                           self.client.url_escape(reason.to_owned()));
        self.client.post_success("/api/report", &body, false)
    }

    fn report_count(&self) -> Option<u64> {
        self.data.num_reports.to_owned()
    }
}

impl<'a> Stickable for Comment<'a> {
    fn stickied(&self) -> bool {
        self.data.stickied
    }

    fn stick(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=yes&sticky=true&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.stickied = true;
        }
        res
    }

    fn unstick(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=no&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.stickied = false;
        }
        res
    }
}

impl<'a> Distinguishable for Comment<'a> {
    fn distinguished(&self) -> Option<String> {
        self.data.distinguished.to_owned()
    }

    fn distinguish(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=yes&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.distinguished = Some(String::from("moderator"));
        }
        res
    }

    fn undistinguish(&mut self) -> Result<(), APIError> {
        let body = format!("api_type=json&how=no&id={}", self.data.name);
        let res = self.client.post_success("/api/distinguish", &body, false);
        if let Ok(()) = res {
            self.data.distinguished = None;
        }
        res
    }
}
