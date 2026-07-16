import os
import json
from google.oauth2 import service_account
from googleapiclient.discovery import build
from googleapiclient.http import MediaFileUpload

def upload_file():
    creds_json = os.environ.get('GDRIVE_CREDENTIALS')
    folder_id = os.environ.get('FOLDER_ID')
    file_path = os.environ.get('FILE_PATH')

    if not creds_json or not folder_id or not file_path:
        print("Missing required environment variables (GDRIVE_CREDENTIALS, FOLDER_ID, FILE_PATH).")
        exit(1)

    try:
        creds_dict = json.loads(creds_json)
    except Exception as e:
        print(f"Error parsing credentials JSON: {e}")
        exit(1)

    try:
        credentials = service_account.Credentials.from_service_account_info(
            creds_dict, scopes=['https://www.googleapis.com/auth/drive.file']
        )
        service = build('drive', 'v3', credentials=credentials)
        
        file_name = os.path.basename(file_path)
        file_metadata = {
            'name': file_name,
            'parents': [folder_id]
        }
        
        media = MediaFileUpload(file_path, resumable=True)
        
        print(f"Uploading {file_name} to Google Drive folder {folder_id}...")
        file = service.files().create(body=file_metadata, media_body=media, fields='id').execute()
        print(f"Successfully uploaded! File ID: {file.get('id')}")
    except Exception as e:
        print(f"Failed to upload to Google Drive: {e}")
        exit(1)

if __name__ == '__main__':
    upload_file()
