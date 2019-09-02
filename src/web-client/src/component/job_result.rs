//! A visual representation of the result of a job.

use crate::model::job::{
    Job,
    Status::{Failed, Succeeded},
};
use dodrio::bumpalo::collections::string::String as BString;
use dodrio::{Node, Render, RenderContext};
use std::marker::PhantomData;

/// The `JobResult` component.
pub(crate) struct JobResult<'a, C> {
    /// A reference to the job for which the results are presented.
    job: &'a Job,

    /// Reference to application controller.
    _controller: PhantomData<C>,
}

impl<'a, C> JobResult<'a, C> {
    /// Create a new `JobResult` component with the provided job reference.
    pub(crate) const fn new(job: &'a Job) -> Self {
        Self {
            job,
            _controller: PhantomData,
        }
    }
}

/// The trait implemented by this component to render all its views.
trait Views<'b> {
    /// The header of the job result.
    fn header(&self, cx: &mut RenderContext<'b>) -> Node<'b>;

    /// The job result output content.
    fn body(&self, cx: &mut RenderContext<'b>) -> Node<'b>;

    /// The staging area for the job result.
    ///
    /// This is a hidden container that contains the raw escaped HTML output.
    /// A separate controller action is responsible for parsing this content and
    /// convert into actual visible HTML on the `body`.
    fn staging(&self, cx: &mut RenderContext<'b>) -> Node<'b>;
}

impl<'a, 'b, C> Views<'b> for JobResult<'a, C> {
    fn header(&self, cx: &mut RenderContext<'b>) -> Node<'b> {
        use dodrio::builder::*;

        let title = match &self.job.status {
            Succeeded(_) => "Success!",
            Failed(_) => "Failed!",
            _ => unreachable!(),
        };

        let title = div(&cx)
            .attr("class", "status")
            .child(div(&cx).child(text(title)).finish())
            .finish();

        header(&cx)
            .child(div(&cx).children([title]).finish())
            .finish()
    }

    fn body(&self, cx: &mut RenderContext<'b>) -> Node<'b> {
        use dodrio::builder::*;

        section(&cx).attr("class", "body").finish()
    }

    fn staging(&self, cx: &mut RenderContext<'b>) -> Node<'b> {
        use dodrio::builder::*;

        let body = match &self.job.status {
            Succeeded(string) | Failed(string) => string,
            _ => unreachable!(),
        };

        let body = BString::from_str_in(body.html.as_ref().unwrap_or(&"".to_owned()), cx.bump)
            .into_bump_str();

        section(&cx)
            .attr("class", "staging")
            .child(text(body))
            .finish()
    }
}

impl<'a, C> Render for JobResult<'a, C> {
    fn render<'b>(&self, cx: &mut RenderContext<'b>) -> Node<'b> {
        use dodrio::builder::*;

        let class = match &self.job.status {
            Succeeded(_) => "job-result success",
            Failed(_) => "job-result failed",
            _ => unreachable!(),
        };

        let class = BString::from_str_in(class, cx.bump).into_bump_str();

        div(&cx)
            .attr("class", class)
            .children([self.header(cx), self.body(cx), self.staging(cx)])
            .finish()
    }
}
