# WebUI Homepage Design Document

## Overview
The homepage of the Beetle web interface should serve as the primary entry point for users to immediately start searching their code indexes. This design focuses on creating a clean, fast, and intuitive search-first experience.

## Design Goals
- **Instant Search**: Users should be able to search immediately upon landing on the homepage
- **Zero Friction**: No login, setup, or navigation required to start searching
- **Visual Clarity**: Clean, minimal design that focuses attention on search
- **Fast Results**: As-you-type search suggestions and instant result loading
- **Index Discovery**: Easy way to see and switch between available indexes

## Homepage Layout

### Hero Section
- **Large Search Bar**: Center-stage search input with placeholder "Search your code..."
- **Index Selector**: Dropdown to select which index to search (defaults to most recent)

### Search Experience
- **Real-time Suggestions**: As-you-type suggestions from index names, file paths, and recent searches
- **Instant Results**: Live-updating results as user types (debounced 300ms)
- **Keyboard Navigation**: Full keyboard support (tab, arrows, enter, escape)
- **Result Previews**: Quick preview of file content with highlighted matches

### Quick Actions
- **Create New Index**: Prominent button for first-time users
- **Manage Indexes**: Link to full index management page
- **Recent Searches**: Carousel of recent searches across all indexes
- **Popular Files**: Most frequently accessed files in selected index

### Empty State Design
When no indexes exist:
- **Welcome Message**: "Welcome to Beetle - Let's get you searching"
- **Setup Guide**: 3-step visual guide to create first index
- **Quick Start**: "Create your first index" primary action button
- **Demo Mode**: Option to try with sample data

### Responsive Design
- **Mobile First**: Single column layout on mobile
- **Tablet**: Two-column layout with sidebar for filters
- **Desktop**: Three-column layout with sidebar, main content, and optional preview panel

## Technical Implementation

### Search API Integration
```
GET /api/search?index={index_name}&q={query}&limit=10&offset=0
```

### Real-time Updates
- Debounced search input (300ms delay)

### Performance Considerations
- Optimistic UI updates

## Visual Design System

### Color Palette
- **Primary**: Blue (#1890ff) - search actions, links
- **Background**: White (#ffffff) - main background
- **Surface**: Gray-50 (#fafafa) - secondary backgrounds
- **Text**: Gray-900 (#111827) - primary text
- **Border**: Gray-200 (#e5e7eb) - borders and dividers

### Typography
- **Headings**: Inter font family, bold weights
- **Body**: Inter font family, regular weights
- **Code**: JetBrains Mono for code snippets
- **Sizes**: Consistent 4px grid system

### Spacing
- **XS**: 4px (micro-interactions)
- **SM**: 8px (small components)
- **MD**: 16px (standard spacing)
- **LG**: 24px (section spacing)
- **XL**: 32px (page spacing)

## User Journey

### New User Flow
1. Land on homepage
2. See empty state with clear call-to-action
3. Guided through creating first index
4. Returned to homepage ready to search
5. Immediate search results appear

### Returning User Flow
1. Land on homepage
2. Previous index selected by default
3. Recent searches displayed
4. Type query and see instant results
5. Quick navigation to specific files

### Power User Flow
1. Land on homepage
2. Use keyboard shortcuts for navigation
3. Switch between indexes quickly
4. Use advanced search operators
5. Bulk operations on results

## Accessibility Features
- **Keyboard Navigation**: Full keyboard support
- **Screen Reader**: Proper ARIA labels and announcements
- **High Contrast**: Support for high contrast mode
- **Font Scaling**: Respects browser font size settings
- **Focus Indicators**: Clear focus states for all interactive elements

## Future Enhancements
- **Search History**: Personal search history with stats
- **Saved Searches**: Ability to save and share searches
- **Team Features**: Shared indexes and search collaboration
- **Advanced Filters**: File size, modification date, author filters
- **Integration**: GitHub/GitLab integration for automatic indexing

## Success Metrics
- **Time to First Search**: < 2 seconds from page load
- **Search Success Rate**: > 80% of searches return relevant results
- **Index Creation Rate**: Track how many users create indexes
- **Return Usage**: Frequency of repeat searches
- **Mobile Usage**: Percentage of mobile searches

## Implementation Phases

### Phase 1: Basic Search (MVP)
- Simple search bar with index selector
- Basic search results
- Empty state handling

### Phase 2: Enhanced UX
- Real-time suggestions
- Result previews
- Keyboard navigation
- Responsive design

### Phase 3: Power Features
- Advanced search operators
- Saved searches
- Search history
- Bulk operations

This design document will evolve as we gather user feedback and implement the features.
