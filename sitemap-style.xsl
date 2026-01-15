<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="2.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:sitemap="http://www.sitemaps.org/schemas/sitemap/0.9"
    xmlns:xhtml="http://www.w3.org/1999/xhtml">

<xsl:output method="html" version="1.0" encoding="UTF-8" indent="yes"/>

<xsl:template match="/">
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>Sitemap</title>
    <style>
        :root {
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #f1f5f9;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --text-muted: #94a3b8;
            --border-color: #e2e8f0;
            --accent-color: #3b82f6;
            --accent-hover: #2563eb;
            --priority-high: #22c55e;
            --priority-medium: #eab308;
            --priority-low: #94a3b8;
        }

        @media (prefers-color-scheme: dark) {
            :root {
                --bg-primary: #0f172a;
                --bg-secondary: #1e293b;
                --bg-tertiary: #334155;
                --text-primary: #f1f5f9;
                --text-secondary: #cbd5e1;
                --text-muted: #64748b;
                --border-color: #334155;
                --accent-color: #60a5fa;
                --accent-hover: #93c5fd;
            }
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background-color: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            padding: 2rem;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
        }

        header {
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }

        h1 {
            font-size: 1.875rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }

        .subtitle {
            color: var(--text-secondary);
            font-size: 0.875rem;
        }

        .stats {
            display: flex;
            gap: 2rem;
            margin-top: 1rem;
            flex-wrap: wrap;
        }

        .stat {
            background: var(--bg-secondary);
            padding: 0.75rem 1.25rem;
            border-radius: 0.5rem;
            border: 1px solid var(--border-color);
        }

        .stat-label {
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-muted);
        }

        .stat-value {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--accent-color);
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1.5rem;
            background: var(--bg-secondary);
            border-radius: 0.5rem;
            overflow: hidden;
            border: 1px solid var(--border-color);
        }

        thead {
            background: var(--bg-tertiary);
        }

        th {
            padding: 0.875rem 1rem;
            text-align: left;
            font-weight: 600;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            border-bottom: 1px solid var(--border-color);
        }

        td {
            padding: 0.875rem 1rem;
            border-bottom: 1px solid var(--border-color);
            font-size: 0.875rem;
        }

        tbody tr:hover {
            background: var(--bg-tertiary);
        }

        tbody tr:last-child td {
            border-bottom: none;
        }

        a {
            color: var(--accent-color);
            text-decoration: none;
            word-break: break-all;
        }

        a:hover {
            color: var(--accent-hover);
            text-decoration: underline;
        }

        .priority {
            display: inline-flex;
            align-items: center;
            gap: 0.375rem;
        }

        .priority-dot {
            width: 0.5rem;
            height: 0.5rem;
            border-radius: 50%;
        }

        .priority-high .priority-dot {
            background: var(--priority-high);
        }

        .priority-medium .priority-dot {
            background: var(--priority-medium);
        }

        .priority-low .priority-dot {
            background: var(--priority-low);
        }

        .changefreq {
            display: inline-block;
            padding: 0.25rem 0.5rem;
            background: var(--bg-tertiary);
            border-radius: 0.25rem;
            font-size: 0.75rem;
            color: var(--text-secondary);
        }

        .date {
            color: var(--text-muted);
            font-size: 0.8125rem;
        }

        footer {
            margin-top: 2rem;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
            text-align: center;
            color: var(--text-muted);
            font-size: 0.75rem;
        }

        @media (max-width: 768px) {
            body {
                padding: 1rem;
            }

            .stats {
                gap: 1rem;
            }

            th, td {
                padding: 0.625rem 0.5rem;
            }

            .hide-mobile {
                display: none;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🗺️ Sitemap</h1>
            <p class="subtitle">This sitemap contains all pages available on this website.</p>
            <div class="stats">
                <div class="stat">
                    <div class="stat-label">Total URLs</div>
                    <div class="stat-value"><xsl:value-of select="count(sitemap:urlset/sitemap:url)"/></div>
                </div>
            </div>
        </header>

        <table>
            <thead>
                <tr>
                    <th>URL</th>
                    <th class="hide-mobile">Priority</th>
                    <th class="hide-mobile">Change Frequency</th>
                    <th class="hide-mobile">Last Modified</th>
                </tr>
            </thead>
            <tbody>
                <xsl:for-each select="sitemap:urlset/sitemap:url">
                    <xsl:sort select="sitemap:priority" order="descending"/>
                    <tr>
                        <td>
                            <a href="{sitemap:loc}"><xsl:value-of select="sitemap:loc"/></a>
                        </td>
                        <td class="hide-mobile">
                            <xsl:choose>
                                <xsl:when test="sitemap:priority &gt;= 0.8">
                                    <span class="priority priority-high">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:when>
                                <xsl:when test="sitemap:priority &gt;= 0.5">
                                    <span class="priority priority-medium">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:when>
                                <xsl:otherwise>
                                    <span class="priority priority-low">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:otherwise>
                            </xsl:choose>
                        </td>
                        <td class="hide-mobile">
                            <xsl:if test="sitemap:changefreq">
                                <span class="changefreq"><xsl:value-of select="sitemap:changefreq"/></span>
                            </xsl:if>
                        </td>
                        <td class="hide-mobile">
                            <xsl:if test="sitemap:lastmod">
                                <span class="date"><xsl:value-of select="sitemap:lastmod"/></span>
                            </xsl:if>
                        </td>
                    </tr>
                </xsl:for-each>
            </tbody>
        </table>

        <footer>
            <p>Generated by Typstify • XML Sitemap Protocol</p>
        </footer>
    </div>
</body>
</html>
</xsl:template>

</xsl:stylesheet>