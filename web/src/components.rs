use libpropagation::citation::Citation;
use topcoat::view::{component, view};

#[component]
pub async fn citation_list(citations: Vec<&Citation>) -> topcoat::Result {
    view! {
        <table>
            <tr>
                <th>"ID"</th>
                <th>"Name"</th>
            </tr>
            for citation in citations {
                <tr>
                    <td>(citation.id)</td>
                    <td>(&citation.title)</td>
                </tr>
            }
        </table>
    }
}
