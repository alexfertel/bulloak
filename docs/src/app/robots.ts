import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
  const url = process.env.NEXT_PUBLIC_SITE_URL;
  return {
    rules: [
      {
        userAgent: "*",
        allow: ["/", "/api/og/*"],
      },
    ],
    sitemap: `${url}/sitemap.xml`,
    host: url,
  };
}
