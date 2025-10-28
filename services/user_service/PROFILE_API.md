# User Profile Management API

## Overview
This document describes the User Profile Management API endpoints implemented for the Anthill multi-tenant SaaS platform.

## Endpoints

### 1. Get Current User Profile
**GET** `/api/v1/users/profile`

Retrieves the complete profile of the authenticated user, including basic user information and extended profile data.

**Authentication:** Required (Bearer token)

**Response:** `200 OK`
```json
{
  "user_id": "uuid",
  "tenant_id": "uuid",
  "email": "user@example.com",
  "full_name": "John Doe",
  "avatar_url": "https://...",
  "phone": "+1234567890",
  "role": "user",
  "email_verified": true,
  "bio": "Software engineer passionate about...",
  "title": "Senior Developer",
  "department": "Engineering",
  "location": "San Francisco, CA",
  "website_url": "https://johndoe.com",
  "social_links": {
    "linkedin": "https://linkedin.com/in/johndoe",
    "github": "https://github.com/johndoe"
  },
  "language": "en",
  "timezone": "America/Los_Angeles",
  "date_format": "YYYY-MM-DD",
  "time_format": "24h",
  "notification_preferences": {
    "email_notifications": true,
    "push_notifications": false,
    "sms_notifications": false,
    "notification_types": {
      "order_updates": true,
      "inventory_alerts": true,
      "system_announcements": true,
      "security_alerts": true,
      "marketing_emails": false
    }
  },
  "profile_visibility": "private",
  "show_email": false,
  "show_phone": false,
  "completeness_score": 85,
  "verified": false,
  "verification_badge": null,
  "custom_fields": {},
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-20T14:45:00Z"
}
```

---

### 2. Update User Profile
**PUT** `/api/v1/users/profile`

Updates the authenticated user's profile information.

**Authentication:** Required (Bearer token)

**Request Body:**
```json
{
  "full_name": "John Doe",
  "phone": "+1234567890",
  "bio": "Updated bio...",
  "title": "Lead Developer",
  "department": "Engineering",
  "location": "New York, NY",
  "website_url": "https://johndoe.com",
  "social_links": {
    "linkedin": "https://linkedin.com/in/johndoe",
    "twitter": "https://twitter.com/johndoe"
  },
  "language": "en",
  "timezone": "America/New_York",
  "date_format": "MM/DD/YYYY",
  "time_format": "12h",
  "notification_preferences": {
    "email_notifications": true,
    "push_notifications": true,
    "sms_notifications": false,
    "notification_types": {
      "order_updates": true,
      "inventory_alerts": true,
      "system_announcements": false,
      "security_alerts": true,
      "marketing_emails": false
    }
  },
  "profile_visibility": "team_only",
  "show_email": true,
  "show_phone": false,
  "custom_fields": {
    "employee_id": "EMP-12345"
  }
}
```

**Note:** All fields are optional. Only provided fields will be updated.

**Response:** `200 OK` - Returns updated profile (same format as GET)

---

### 3. Update Profile Visibility Settings
**PUT** `/api/v1/users/profile/visibility`

Updates the visibility settings for the user's profile.

**Authentication:** Required (Bearer token)

**Request Body:**
```json
{
  "profile_visibility": "public",
  "show_email": true,
  "show_phone": false
}
```

**Profile Visibility Options:**
- `public` - Profile visible to all users in the tenant
- `private` - Profile only visible to the user
- `team_only` - Profile visible to team members only

**Response:** `200 OK`

---

### 4. Get Profile Completeness Score
**GET** `/api/v1/users/profile/completeness`

Retrieves the profile completeness score and suggestions for improvement.

**Authentication:** Required (Bearer token)

**Response:** `200 OK`
```json
{
  "score": 75,
  "missing_fields": [
    "avatar_url",
    "phone",
    "bio"
  ],
  "suggestions": [
    "Upload a profile picture to personalize your account",
    "Add your phone number for better account security",
    "Add a bio to tell others about yourself"
  ]
}
```

**Scoring System:**
- Full name: 15 points
- Avatar: 15 points
- Phone: 10 points
- Email verified: 20 points
- Bio: 10 points
- Title: 10 points
- Department: 5 points
- Location: 5 points
- Social links: 10 points
- **Total:** 100 points

---

### 5. Search Profiles
**POST** `/api/v1/users/profiles/search`

Search for user profiles within the tenant.

**Authentication:** Required (Bearer token)

**Request Body:**
```json
{
  "query": "john",
  "department": "Engineering",
  "location": "San Francisco",
  "verified_only": false,
  "page": 1,
  "per_page": 20
}
```

**Query Parameters (all optional):**
- `query` - Search term (searches in name, title, bio)
- `department` - Filter by department
- `location` - Filter by location
- `verified_only` - Only show verified profiles
- `page` - Page number (default: 1)
- `per_page` - Results per page (default: 20, max: 100)

**Response:** `200 OK`
```json
{
  "profiles": [
    {
      "user_id": "uuid",
      "full_name": "John Doe",
      "avatar_url": "https://...",
      "title": "Senior Developer",
      "department": "Engineering",
      "location": "San Francisco, CA",
      "bio": "Software engineer...",
      "verified": true,
      "verification_badge": "trusted",
      "social_links": {
        "linkedin": "https://linkedin.com/in/johndoe"
      }
    }
  ],
  "total": 42
}
```

---

### 6. Get Public Profile
**GET** `/api/v1/users/profiles/{user_id}`

Retrieves the public profile of a specific user (only if profile is public).

**Authentication:** Required (Bearer token)

**Path Parameters:**
- `user_id` - UUID of the user

**Response:** `200 OK`
```json
{
  "user_id": "uuid",
  "full_name": "John Doe",
  "avatar_url": "https://...",
  "title": "Senior Developer",
  "department": "Engineering",
  "location": "San Francisco, CA",
  "bio": "Software engineer...",
  "verified": true,
  "verification_badge": "trusted",
  "social_links": {
    "linkedin": "https://linkedin.com/in/johndoe"
  }
}
```

**Error Responses:**
- `403 Forbidden` - Profile is not public
- `404 Not Found` - User or profile not found

---

### 7. Update Profile Verification (Admin Only)
**PUT** `/api/v1/users/profiles/{user_id}/verification`

Updates the verification status of a user's profile. This endpoint is restricted to admin users.

**Authentication:** Required (Bearer token with admin role)

**Path Parameters:**
- `user_id` - UUID of the user

**Request Body:**
```json
{
  "verified": true,
  "badge": "trusted"
}
```

**Badge Options:**
- `verified` - Basic verification
- `trusted` - Trusted user
- `expert` - Expert/specialist
- `null` - No badge

**Response:** `200 OK`

**Error Responses:**
- `403 Forbidden` - User is not an admin
- `404 Not Found` - User or profile not found

---

## Database Schema

### user_profiles Table
```sql
CREATE TABLE user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL UNIQUE,
    tenant_id UUID NOT NULL,
    
    -- Extended profile
    bio TEXT,
    title VARCHAR(255),
    department VARCHAR(255),
    location VARCHAR(255),
    website_url TEXT,
    
    -- JSON fields
    social_links JSONB DEFAULT '{}',
    notification_preferences JSONB DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    
    -- Preferences
    language VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(100) DEFAULT 'UTC',
    date_format VARCHAR(50) DEFAULT 'YYYY-MM-DD',
    time_format VARCHAR(50) DEFAULT '24h',
    
    -- Privacy
    profile_visibility VARCHAR(50) DEFAULT 'private',
    show_email BOOLEAN DEFAULT FALSE,
    show_phone BOOLEAN DEFAULT FALSE,
    
    -- Completeness
    completeness_score INTEGER DEFAULT 0,
    last_completeness_check_at TIMESTAMPTZ,
    
    -- Verification
    verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    verification_badge VARCHAR(50),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Security Considerations

1. **Tenant Isolation**: All operations are tenant-scoped. Users can only access profiles within their tenant.

2. **Authentication**: All endpoints require valid JWT authentication.

3. **Authorization**: 
   - Regular users can only view/edit their own profile
   - Admin users can verify profiles and view all profiles in the tenant

4. **Privacy Settings**: Profile visibility controls who can see user information.

5. **Data Validation**: All input is validated and sanitized before storage.

## Implementation Notes

- Profile is automatically created when a user registers (via database trigger)
- Profile completeness is calculated using a PostgreSQL function
- Avatar upload is currently a placeholder (requires S3 integration)
- All JSONB fields support flexible extensibility
- Supports GDPR compliance through privacy settings
