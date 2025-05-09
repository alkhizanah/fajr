use limine::BaseRevision;
use limine::request::{
    FramebufferRequest, RequestsEndMarker, RequestsStartMarker, StackSizeRequest,
};

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(crate::STACK_SIZE);

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
