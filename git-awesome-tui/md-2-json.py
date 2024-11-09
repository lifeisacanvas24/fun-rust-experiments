import json
import logging
import re
from urllib.parse import urlparse

import requests

logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')

# Validate URL function
def validate_url(url):
    """Check if the URL is well-formed and either valid or a fragment link."""
    parsed_url = urlparse(url)
    # Accepts both HTTP/HTTPS URLs and fragment links (e.g., '#section')
    return parsed_url.scheme in ['http', 'https'] or bool(parsed_url.fragment)

def parse_markdown(md_content):
    print("Starting Markdown parsing...")
    data = []
    current_category = None
    current_subcategory = None

    lines = md_content.splitlines()
    print(f"Total lines to process: {len(lines)}")

    for i, line in enumerate(lines):
        line = line.strip()
        if not line or line.startswith('<'):
            continue  # Skip empty lines and lines with HTML content

        if re.match(r'##\s*Contents', line):
            print(f"Skipping 'Contents' header on line {i + 1}.")
            continue

        if line.startswith("## "):  # Category header
            title = line[3:].strip()  # Removing "## "
            print(f"Detected category: {title}")

            if current_category:
                data.append(current_category)

            # Initialize category structure
            current_category = {
                "title": title,
                "subcategories": [],
            }
            current_subcategory = None

        elif line.startswith("- "):  # Subcategory or link
            subcategory_title = line[2:].strip()
            print(f"Detected subcategory: {subcategory_title}")

            # Check if the subcategory is a link
            link_match = re.match(r"- \[(.*?)\]\((.*?)\)", line)
            if link_match:
                link_title = link_match.group(1)
                link_url = link_match.group(2)

                # Ensure current_category exists before appending
                if current_category is None:
                    current_category = {
                        "title": "Uncategorized",
                        "subcategories": [],
                    }

                if current_subcategory is None:
                    # Create new subcategory
                    current_subcategory = {
                        "title": subcategory_title,
                        "links": [{"title": link_title, "url": link_url}],
                    }
                    current_category["subcategories"].append(current_subcategory)
                else:
                    current_subcategory["links"].append({"title": link_title, "url": link_url})
            else:
                # If no link, just add the subcategory title
                current_subcategory = {
                    "title": subcategory_title,
                    "links": [],
                }
                current_category["subcategories"].append(current_subcategory)

        else:
            link_match = re.match(r"- \[(.*?)\]\((.*?)\)", line)
            if link_match:
                link_title = link_match.group(1)
                link_url = link_match.group(2)

                # Validate the URL and add it even if it's a fragment link
                if validate_url(link_url):
                    if current_subcategory:
                        current_subcategory["links"].append({"title": link_title, "url": link_url})
                    elif current_category:
                        # Append the link to the last subcategory
                        if current_category["subcategories"]:
                            current_category["subcategories"][-1]["links"].append({"title": link_title, "url": link_url})

    # Append the last category if it exists
    if current_category:
        data.append(current_category)

    print("Markdown parsing completed.")
    return data

def fetch_readme():
    url = 'https://raw.githubusercontent.com/sindresorhus/awesome/master/readme.md'
    print(f"Fetching README from {url}...")
    response = requests.get(url)
    if response.status_code == 200:
        print("README fetched successfully.")
        return response.text
    print(f"Failed to fetch README, status code: {response.status_code}")
    return None

def save_json(data):
    with open("awesome.json", "w") as f:
        json.dump(data, f, indent=4)
    print("Data saved successfully.")

if __name__ == "__main__":
    md_content = fetch_readme()
    if md_content:
        parsed_data = parse_markdown(md_content)
        save_json(parsed_data)
