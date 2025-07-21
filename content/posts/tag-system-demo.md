---
title: Tag System Demo Post
author: Developer
date: 2025-07-21
tags: rust, ipfs, blog, features
category: Development
excerpt: Demonstrating the new tag system in KPGB
---

# Tag System Demo Post

This post demonstrates the new tag system implementation in KPGB.

## Features Added

The tag system now includes:

1. **Tag Browsing**: Click on any tag to see all posts with that tag
2. **Tag Cloud**: A dedicated tags page showing all tags with post counts
3. **API Support**: New endpoints for tag-related queries
4. **Static Generation**: Tag pages are generated for static sites
5. **Clean URLs**: Tag pages use clean URL structure

## Technical Implementation

- Database layer: Tag queries with post counts
- API endpoints: `/api/tags` and `/api/tags/:tag`
- Web handlers: Tag listing and filtered post views
- Templates: New tag cloud and tag posts pages
- CSS styling: Enhanced tag presentation

Try clicking on the tags below to explore posts by topic!