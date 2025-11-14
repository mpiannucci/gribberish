import os
import sys
sys.path.insert(0, os.path.abspath('../../python'))

# Project information
project = 'gribberish'
copyright = '2024, Matthew Iannucci'
author = 'Matthew Iannucci'
version = '0.3.0'  # Update to match your version

# Extensions
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',
    'sphinx.ext.viewcode',
    'sphinx.ext.intersphinx',
    'sphinx.ext.autosummary',
    'myst_parser',
    'sphinx_copybutton',  # Adds copy button to code blocks
    'sphinx_design',  # For cards, grids, tabs, etc.
]

# File patterns
source_suffix = {
    '.rst': 'restructuredtext',
    '.md': 'markdown',
}
master_doc = 'index'

# -- PyData Theme Options -----------------------------------------------------

html_theme = 'pydata_sphinx_theme'

html_theme_options = {
    "github_url": "https://github.com/mpiannucci/gribberish",
    "show_toc_level": 2,
    "navbar_align": "left",
    
    # Add your logo if you have one
    # "logo": {
    #     "image_light": "_static/logo-light.png",
    #     "image_dark": "_static/logo-dark.png",
    # },
    
    # Header links
    "header_links_before_dropdown": 4,
    "icon_links": [
        {
            "name": "GitHub",
            "url": "https://github.com/mpiannucci/gribberish",
            "icon": "fa-brands fa-github",
            "type": "fontawesome",
        },
        {
            "name": "PyPI",
            "url": "https://pypi.org/project/gribberish",
            "icon": "fa-brands fa-python",
            "type": "fontawesome",
        },
    ],
    
    # Navigation
    "navigation_with_keys": True,
    "show_nav_level": 2,
    "show_prev_next": True,
    
    # Search bar location ("navbar" or "sidebar")
    "search_bar_position": "navbar",
    
    # Footer
    "footer_start": ["copyright"],
    "footer_end": ["last-updated"],
    
    # Sidebar
    "sidebar_includehidden": True,
    "use_sidenotes": True,
    
    # Version switcher (if you want to support multiple versions)
    # "switcher": {
    #     "json_url": "https://mpiannucci.github.io/gribberish/versions.json",
    #     "version_match": version,
    # },
}

html_sidebars = {
    "**": ["sidebar-nav-bs", "sidebar-ethical-ads"],
    "index": [],  # Remove sidebar from landing page
}

# Add any paths that contain custom static files
html_static_path = ['_static']
html_css_files = [
    'css/custom.css',  # Optional: for custom styling
]

# Favicon
# html_favicon = "_static/favicon.ico"

# -- Autodoc configuration ---------------------------------------------------

autodoc_default_options = {
    'members': True,
    'member-order': 'bysource',
    'special-members': '__init__',
    'undoc-members': True,
    'exclude-members': '__weakref__'
}

# Napoleon settings for Google and NumPy style docstrings
napoleon_google_docstring = False
napoleon_numpy_docstring = True
napoleon_use_param = True
napoleon_use_rtype = True

# -- Intersphinx configuration ----------------------------------------------

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
    'numpy': ('https://numpy.org/doc/stable/', None),
    'xarray': ('https://docs.xarray.dev/en/stable/', None),
    'pandas': ('https://pandas.pydata.org/docs/', None),
}

# -- MyST configuration -----------------------------------------------------

myst_enable_extensions = [
    "colon_fence",    # ::: fence syntax
    "deflist",        # Definition lists
    "html_image",     # HTML images
    "linkify",        # Auto-detect links
    "replacements",   # Text replacements
    "smartquotes",    # Smart quotes
    "tasklist",       # - [ ] task lists
    "attrs_inline",   # Inline attributes
]

# -- Copy button configuration ----------------------------------------------

copybutton_prompt_text = r">>> |\.\.\. |\$ |In \[\d*\]: | {2,5}\.\.\.: | {5,8}: "
copybutton_prompt_is_regexp = True
copybutton_line_continuation_character = "\\"