// PDF Template JavaScript

// Tối ưu cho in PDF - Chỉ áp dụng cho PDF template
window.addEventListener('load', function() {
    // Tự động mở tất cả các phần tử details cho PDF
    const detailsElements = document.querySelectorAll('.pdf-template details');
    detailsElements.forEach(details => {
        details.setAttribute('open', 'true');
    });
    
    // Thêm class avoid-break cho các section
    const sections = document.querySelectorAll('.pdf-template section, .pdf-template .report-section');
    sections.forEach(section => {
        section.classList.add('avoid-break');
    });
    
    // Auto print function (commented out by default)
    // setTimeout(function() {
    //     adjustFontForPrint();
    //     window.print();
    // }, 2000);
});

// Tối ưu trước khi in
window.addEventListener('beforeprint', function() {
    // Đảm bảo tất cả details đều được mở trước khi in
    const detailsElements = document.querySelectorAll('.pdf-template details');
    detailsElements.forEach(details => {
        details.setAttribute('open', 'true');
    });
    
    // Ẩn các element không cần thiết
    const elementsToHide = document.querySelectorAll('.pdf-template .sidebar, .pdf-template .navigation, .pdf-template .breadcrumb');
    elementsToHide.forEach(el => el.style.display = 'none');
});

// Khôi phục sau khi in
window.addEventListener('afterprint', function() {
    const elementsToShow = document.querySelectorAll('.pdf-template .sidebar, .pdf-template .navigation, .pdf-template .breadcrumb');
    elementsToShow.forEach(el => el.style.display = '');
});
