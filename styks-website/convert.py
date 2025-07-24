import re
import markdown
from bs4 import BeautifulSoup

INPUT_MD = "../README.md"
OUTPUT_HTML = "http-content/index.html"

def post_process_html(html_content: str) -> str:
    """Post-process HTML to add proper classes and improve styling"""
    soup = BeautifulSoup(html_content, 'html.parser')
    
    # Find all code blocks and ensure they have proper highlighting classes
    for pre in soup.find_all('pre'):
        if not pre.get('class'):
            pre['class'] = ['highlight']
    
    # Process inline code elements
    for code in soup.find_all('code'):
        if not code.parent or code.parent.name != 'pre':
            if not code.get('class'):
                code['class'] = ['highlighter-rouge']
    
    # Add clearfix class to container elements that might need it
    for div in soup.find_all('div', class_='mermaid'):
        if 'cf' not in div.get('class', []):
            div['class'] = div.get('class', []) + ['cf']
    

    return str(soup)

def md_with_mermaid_to_html(md_text: str) -> str:
    # Replace fenced mermaid blocks with <div class="mermaid">…</div>
    pattern = re.compile(r"```mermaid\s+(.*?)```", re.DOTALL)
    def repl(m):
        inner = m.group(1).strip()
        return f'<div class="mermaid">\n{inner}\n</div>'
    md_text = pattern.sub(repl, md_text)

    # Convert the remaining markdown to HTML with better extensions
    body = markdown.markdown(
        md_text,
        extensions=[
            "fenced_code", 
            "codehilite",
            "tables",
            "toc"
        ]
    )
    
    # Post-process the body content
    body = post_process_html(body)

    # Load the template HTML
    with open("template.html", "r", encoding="utf-8") as f:
        template = f.read()

    # Insert the body into the template
    template = template.replace("<!-- BODY -->", body)

    # Final HTML document including Mermaid script with proper structure
    return template

if __name__ == "__main__":
    with open(INPUT_MD, "r", encoding="utf-8") as f:
        md_text = f.read()

    # Take only the content between <!-- WEBSITE: START --> and <!-- WEBSITE: END -->
    start_marker = "<!-- WEBSITE: START -->"
    end_marker = "<!-- WEBSITE: END -->"
    start_index = md_text.find(start_marker) + len(start_marker)
    end_index = md_text.find(end_marker, start_index)
    if start_index == -1 or end_index == -1:
        raise ValueError("Markers not found in the markdown file.")
    md_text = md_text[start_index:end_index].strip()

    # Remove all lines that start with "TODO:"
    md_text = "\n".join(line for line in md_text.splitlines() if not line.startswith("TODO:"))

    html = md_with_mermaid_to_html(md_text)

    with open(OUTPUT_HTML, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"Wrote {OUTPUT_HTML}")
