import type { MetadataRoute } from "next";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "ФСП Таланты",
    short_name: "ФСП Таланты",
    description: "Подбор ИТ-специалистов Федерации спортивного программирования России: один тест, и компании пишут вам сами.",
    lang: "ru",
    start_url: "/",
    display: "standalone",
    background_color: "#ffffff",
    theme_color: "#402fff",
    icons: [
      { src: "/icons/icon-192.png", sizes: "192x192", type: "image/png", purpose: "any" },
      { src: "/icons/icon-512.png", sizes: "512x512", type: "image/png", purpose: "any" },
      { src: "/icons/icon-maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
    ],
  };
}
