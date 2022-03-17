require "bundler/inline"

gemfile do
  source "https://rubygems.org"

  git_source(:github) { |repo_name| "https://github.com/#{repo_name}" }

  gem "google-api-client"
end

ENV["GOOGLE_APPLICATION_CREDENTIALS"] = "credentials.json"

require 'google/apis/drive_v3'
Google::Apis.logger.level = Logger::DEBUG

authorizer = Google::Auth::ServiceAccountCredentials.make_creds(
  json_key_io: File.open("credentials.json"),
  scope: %w(
   https://www.googleapis.com/auth/drive
   https://www.googleapis.com/auth/drive.file
  )
)

authorizer.fetch_access_token!
drive = Google::Apis::DriveV3::DriveService.new
drive.authorization = authorizer

# pp drive

using Module.new {
  # refine Google::Apis::DriveV3::DriveService do
  #   def create_file(file_object = nil, enforce_single_parent: nil, ignore_default_visibility: nil, include_permissions_for_view: nil, keep_revision_forever: nil, ocr_language: nil, supports_all_drives: nil, supports_team_drives: nil, use_content_as_indexable_text: nil, fields: nil, quota_user: nil, user_ip: nil, upload_source: nil, content_type: nil, options: nil, &block)

  #     if upload_source.nil?
  #       command = make_simple_command(:post, 'files', options)
  #     else
  #       command = make_upload_command(:post, 'files', options)
  #       command.upload_source = upload_source
  #       command.upload_content_type = content_type
  #     end
  #     command.request_representation = Google::Apis::DriveV3::File::Representation
  #     command.request_object = file_object
  #     command.response_representation = Google::Apis::DriveV3::File::Representation
  #     command.response_class = Google::Apis::DriveV3::File
  #     command.query['enforceSingleParent'] = enforce_single_parent unless enforce_single_parent.nil?
  #     command.query['ignoreDefaultVisibility'] = ignore_default_visibility unless ignore_default_visibility.nil?
  #     command.query['includePermissionsForView'] = include_permissions_for_view unless include_permissions_for_view.nil?
  #     command.query['keepRevisionForever'] = keep_revision_forever unless keep_revision_forever.nil?
  #     command.query['ocrLanguage'] = ocr_language unless ocr_language.nil?
  #     command.query['supportsAllDrives'] = supports_all_drives unless supports_all_drives.nil?
  #     command.query['supportsTeamDrives'] = supports_team_drives unless supports_team_drives.nil?
  #     command.query['useContentAsIndexableText'] = use_content_as_indexable_text unless use_content_as_indexable_text.nil?
  #     command.query['fields'] = fields unless fields.nil?
  #     command.query['quotaUser'] = quota_user unless quota_user.nil?
  #     command.query['userIp'] = user_ip unless user_ip.nil?
  #     pp command.body
  #     pp command.header
  #     super
  #   end
  # end
  refine Google::Apis::Core::ResumableUploadCommand do
    def send_start_command(client)
      pp "Call!!", client
      super
    end
  end
}

res = drive.create_file(
  {
    name: "text-ruby.txt",
    parents: ["1CcB4hzyRqmSKpviFV0vCp0QEi88V9zEt"],
    mimeType: "text/plain",
  },
  upload_source: "./test.txt",
)

pp res
