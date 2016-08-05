use structures::comment_list::CommentList;
use structures::submission::FlairList;
use structures::user::User;
use structures::subreddit::Subreddit;
use structures::comment::Comment;
use errors::APIError;

/// An object that can be voted upon and has a score based on the upvotes - downvotes.
/// ## Notes
/// The `ups` and `downs` values from the API no longer represent the true upvotes and downvotes,
/// so this trait does not expose them.
pub trait Votable {
    /// The (fuzzed) points score of the object.
    fn score(&self) -> i64;
    /// Indicates the logged-in user's current vote on this object:
    /// - Some(true) = Upvoted
    /// - Some(false) = Downvoted
    /// - None = No vote
    fn likes(&self) -> Option<bool>;
    /// Upvotes the specified post, if possible.
    fn upvote(&self) -> Result<(), APIError>;
    /// Downvotes the specified post, if possible.
    fn downvote(&self) -> Result<(), APIError>;
    /// Removes the vote on the specified post, if possible.
    fn cancel_vote(&self) -> Result<(), APIError>;
}

/// A paginatable listing.
pub trait PageListing {
    /// The ID to use for anchoring when paginating to the previous page.
    fn before(&self) -> Option<String>;
    /// The ID to use for anchoring when paginating to the next page.
    fn after(&self) -> Option<String>;
    /// The modhash (CSRF token) sent with this listing. Largely redundant, since the main
    /// modhash can be used.
    fn modhash(&self) -> Option<String>;
}

/// An object that was created at some point (e.g. a subreddit, a submission or a comment)
pub trait Created {
    /// The timestamp of the time when the post was created, as would be shown to the logged-in
    /// user.
    fn created(&self) -> i64;
    /// The timestamp of post creation, in UTC.
    fn created_utc(&self) -> i64;
}

/// An object that can be edited (anything that has a body).
pub trait Editable {
    /// `true` if edited, otherwise `false`.
    fn edited(&self) -> bool;
    /// Returns `Some(edited timestamp in logged-in user's time zone)` if edited, otherwise `None`.
    fn edited_time(&self) -> Option<i64>;
    /// Edits the specified post (if possible) with the new text (in **Markdown** format)
    fn edit(&mut self, text: &str) -> Result<(), APIError>;
    /// Gets the body of a comment or message or the self text of a post, if available.
    /// In the case of link posts, this will be `None`.
    fn body(&self) -> Option<String>;
    /// Gets the comment/message body HTML or self text HTML if available.
    fn body_html(&self) -> Option<String>;
    // TODO: anything editable has a body: refactor to handle this!
}

/// An object that was created by an author and is in a subreddit (i.e. a submission or comment)
pub trait Content {
    /// The author of the object.
    fn author(&self) -> User;
    /// The flair text of the user flair, if present.
    fn author_flair_text(&self) -> Option<String>;
    /// The flair CSS class of the user flair, if present.
    fn author_flair_css(&self) -> Option<String>;
    /// For submissions (link/self posts), this is the subreddit where it was posted. For comments,
    /// this is the subreddit of the parent submission.
    fn subreddit(&self) -> Subreddit;
    /// Deletes the specified object, if possible. **This may be irreversible. Use with caution.**
    fn delete(self) -> Result<(), APIError>;
    /// Gets the full ID of this comment (kind + id)
    fn name(&self) -> &str;
}

/// An object that can be approved or removed by a moderator.
pub trait Approvable {
    /// Approves the message, clearing any previous reports of the message. If this was removed
    /// (either by a moderator or the spam filter), the item will be restored to the appropriate
    /// listings.
    fn approve(&self) -> Result<(), APIError>;
    /// Removes the message so that it is no longer visible in listings. If the spam parameter is
    /// set to true, this will be flagged for the site-wide spam filter.
    fn remove(&self, spam: bool) -> Result<(), APIError>;
    /// Ignores reports on this item so they do not appear in this subreddit's modmail.
    fn ignore_reports(&self) -> Result<(), APIError>;
    /// Stops ignoring reports on this item, so they appear in the modmail once again.
    fn unignore_reports(&self) -> Result<(), APIError>;
}

/// An object that can be commented upon and may have comments.
pub trait Commentable<'a> {
    /// The number of comments on this object. Prefer this to `replies().count()`.
    fn reply_count(&self) -> u64;
    /// Sends a reply with the specified body.
    fn reply(&self, &str) -> Result<Comment, APIError>;
    /// Gets all replies as a self-paginating `CommentList`, which can be iterated through as
    /// necessary. Comments cannot be batched like submission listings, so there may be
    /// multiple requests on large threads to get all comments.
    fn replies(self) -> Result<CommentList<'a>, APIError>;
}


/// An object that can be stickied (made into an 'annoucement post'). In practice, this is only
/// self/link posts.
pub trait Stickable {
    /// Returns the **current** sticky state of the submission.
    fn stickied(&self) -> bool;
    /// Makes the selected post a sticky, provided that you have the correct privileges.
    /// For comments, this will also distinguish the post as [M].
    fn stick(&mut self) -> Result<(), APIError>;
    /// Makes this sticky post a normal post, provided that you have the correct privileges.
    /// For comments, this will also remove the [M] distinguish.
    fn unstick(&mut self) -> Result<(), APIError>;
    /// Toggles the sticky state (i.e. becomes sticky if it is not one, and becomes normal if it
    /// is a sticky)
    fn toggle_sticky(&mut self) -> Result<(), APIError> {
        if self.stickied() {
            self.unstick()
        } else {
            self.stick()
        }
    }
}

/// An object that can be locked so that no further comments can be added.
pub trait Lockable {
    /// Returns the **current** locked state of the submission.
    fn locked(&self) -> bool;
    /// Locks the current submission, provided you have the correct privileges.
    fn lock(&mut self) -> Result<(), APIError>;
    /// Unlocks the current submission, provided you have the correct privileges.
    fn unlock(&mut self) -> Result<(), APIError>;
    /// Toggles the lock state (locks if unlocked, unlocks if locked).
    fn toggle_lock(&mut self) -> Result<(), APIError> {
        if self.locked() {
            self.unlock()
        } else {
            self.lock()
        }
    }
}

/// An object that can be reported (submission, comment, private message). Also exposes moderation
/// options to review reports.
pub trait Reportable {
    /// Reports the object for the specified reason (must be less than 100 characters).
    fn report(&self, reason: &str) -> Result<(), APIError>;
    /// Gets the number of reports for this post. If you do not have the rights to view this value,
    /// this will return `None`.
    fn report_count(&self) -> Option<u64>;
    // TODO: add report_reasons
}

/// An object that has a flair and can be given a flair by the creator or a moderator.
pub trait Flairable {
    /// Gets the current flair text, if present.
    fn get_flair_text(&self) -> Option<String>;
    /// Gets the current CSS class of the flair, if present.
    fn get_flair_css(&self) -> Option<String>;
    /// Tries to fetch the flair options for this item. If you do not have the privileges to add
    /// a flair to this (i.e. not moderator or author), this will give an
    /// `APIError::HTTPError(Forbidden)`.
    fn flair_options(&self) -> Result<FlairList, APIError>;
    /// Sets the flair for this post, if possible. The `template` parameter is the
    /// `FlairChoice.flair_template_id` field. The template will also be returned if using
    /// `FlairList.find_text`. This may fail with `APIError::HTTPError(Forbidden)` if you are
    /// not authorized to do this.
    fn flair(&self, template: &str) -> Result<(), APIError>;
}

/// An object that can be shown/hidden in listings.
pub trait Visible {
    /// Gets the current visibility state of the ooject.
    fn hidden(&self) -> bool;
    /// Hides the object, so it is not shown in listings. May be useful so that bots do not
    /// view the same links multiple times.
    fn hide(&mut self) -> Result<(), APIError>;
    /// Shows the object again, so it **is** visible in listings.
    fn show(&mut self) -> Result<(), APIError>;
    /// Hides the object if visible, and shows it if hidden.
    fn toggle_hidden(&mut self) -> Result<(), APIError> {
        if self.hidden() {
            self.show()
        } else {
            self.hide()
        }
    }
}

/// An object that can be distinguished (moderator/admin/special indicator).
pub trait Distinguishable {
    /// Indicates whether the user has used a special flag for themselves, e.g. [M] or [A].
    /// Possible values:
    ///
    /// - `None` - Normal user
    /// - `Some("moderator")` - [M]
    /// - `Some("admin")` - [A]
    /// - `Some("special")` - other special 'distinguishes' e.g. [Δ]
    fn distinguished(&self) -> Option<String>;
    /// Sets the post to have a [M] distinguish.
    fn distinguish(&mut self) -> Result<(), APIError>;
    /// Removes any distinguish on the comment. This will also unsticky a comment, if it is
    /// stickied.
    fn undistinguish(&mut self) -> Result<(), APIError>;
    /// Distinguishes if undistinguished, and vice versa.
    fn toggle_distinguish(&mut self) -> Result<(), APIError> {
        if let Some(_) = self.distinguished() {
            self.undistinguish()
        } else {
            self.distinguish()
        }
    }
}
