import re
import markdown

INPUT_MD = "arch.md"
OUTPUT_HTML = "arch.html"

def md_with_mermaid_to_html(md_text: str) -> str:
    # Replace fenced mermaid blocks with <div class="mermaid">…</div>
    pattern = re.compile(r"```mermaid\s+(.*?)```", re.DOTALL)
    def repl(m):
        inner = m.group(1).strip()
        return f'<div class="mermaid">\n{inner}\n</div>'
    md_text = pattern.sub(repl, md_text)

    # Convert the remaining markdown to HTML
    body = markdown.markdown(
        md_text,
        extensions=["fenced_code", "codehilite"]  # optional highlighting
    )

    # Final HTML document including Mermaid script
    return f"""<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>arch</title>
<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
<script>mermaid.initialize({{ startOnLoad: true }});</script>
</head>
<body>
{body}
</body>
</html>"""

if __name__ == "__main__":
    with open(INPUT_MD, "r", encoding="utf-8") as f:
        md_text = f.read()

    html = md_with_mermaid_to_html(md_text)

    with open(OUTPUT_HTML, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"Wrote {OUTPUT_HTML}")
