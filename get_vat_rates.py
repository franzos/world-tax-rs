import json
import tempfile
import urllib.request
from typing import Dict

# Source URLs
FORMAT1_URL = "https://github.com/valeriansaliou/node-sales-tax/raw/master/res/sales_tax_rates.json"
FORMAT2_URL = "https://github.com/benbucksch/eu-vat-rates/raw/master/rates.json"
OUTPUTT_FILE = "vat_rates.json"

def fetch_to_temp(url: str) -> str:
    """Fetch JSON from URL and save to temp file"""
    temp = tempfile.NamedTemporaryFile(delete=False)
    urllib.request.urlretrieve(url, temp.name)
    return temp.name

def convert_format1(data: Dict) -> Dict:
    """Convert Format 1 to unified format"""
    result = {}
    for country_code, country_data in data.items():
        converted = {
            "type": country_data.get("type", "none"),
            "currency": country_data.get("currency", ""),
            "standard_rate": country_data.get("rate", 0)
        }
        
        # Handle states if present
        if "states" in country_data:
            converted["states"] = {
                state_code: {
                    "standard_rate": state_data.get("rate", 0),
                    "type": state_data.get("type", "none")
                }
                for state_code, state_data in country_data["states"].items()
            }
            
        result[country_code] = converted
    return result

def convert_format2(data: Dict) -> Dict:
    """Convert Format 2 to unified format"""
    result = {}
    for country_code, country_data in data["rates"].items():
        result[country_code] = {
            "type": "vat",
            "currency": "EUR",
            "standard_rate": country_data.get("standard_rate", 0) / 100,
            "reduced_rate": country_data.get("reduced_rate", 0) / 100,
            "reduced_rate_alt": country_data.get("reduced_rate_alt", 0) / 100,
            "super_reduced_rate": country_data.get("super_reduced_rate", 0) / 100,
            "parking_rate": country_data.get("parking_rate", 0) / 100,
            "vat_name": country_data.get("vat_name", ""),
            "vat_abbr": country_data.get("vat_abbr", "")
        }
    return result

def main():
    # 1. Fetch from URLs to temp files
    temp_file1 = fetch_to_temp(FORMAT1_URL)
    temp_file2 = fetch_to_temp(FORMAT2_URL)
    
    # 2. Read from temp files and 3. Parse JSON
    with open(temp_file1) as f:
        data1 = json.load(f)
    with open(temp_file2) as f:
        data2 = json.load(f)
    
    # Convert both formats to unified format
    converted1 = convert_format1(data1)
    converted2 = convert_format2(data2)
    
    # 3. Merge and deduplicate (Format 2 overwrites Format 1)
    merged = {**converted1, **converted2}
    
    # 4. Write to output file
    with open(OUTPUTT_FILE, "w") as f:
        json.dump(merged, f, indent=2)

if __name__ == "__main__":
    main()
